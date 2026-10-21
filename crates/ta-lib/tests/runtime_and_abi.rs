//! Runtime state and native abstract-API parity against the linked TA-Lib library.

use std::{
    collections::BTreeSet,
    ffi::{CStr, CString},
    ptr,
    sync::{Mutex, MutexGuard},
};

const TA_SUCCESS: sys::TA_RetCode = sys::TA_RetCode::TA_SUCCESS;
use anyhow::Context;
use ta_lib::{self as ta, CandleSettingType as Candle, Compatibility, RangeType, sys};

static RUNTIME: Mutex<()> = Mutex::new(());

struct RuntimeGuard(MutexGuard<'static, ()>);
impl RuntimeGuard {
    fn new() -> Self {
        let lock = RUNTIME.lock().expect("runtime test mutex poisoned");
        ta::initialize().context("initialize TA-Lib").unwrap();
        Self(lock)
    }
}
impl Drop for RuntimeGuard {
    fn drop(&mut self) {
        // Keep the lock alive until global defaults have been restored.
        let _lock = &self.0;
        ta::shutdown().expect("shutdown TA-Lib");
    }
}

#[test]
fn lifecycle_repeated_initialize_and_shutdown_restore_defaults() {
    let _runtime = RuntimeGuard::new();
    ta::initialize().unwrap();
    ta::set_unstable_period(ta::FuncUnstId::TA_FUNC_UNST_EMA, 17).unwrap();
    ta::shutdown().unwrap();
    ta::shutdown().unwrap();
    ta::initialize().unwrap();
    assert_eq!(ta::get_unstable_period(ta::FuncUnstId::TA_FUNC_UNST_EMA), 0);
    assert_eq!(ta::get_compatibility(), Compatibility::Default);
}

#[test]
fn version_components_match_native_static_strings() {
    let _runtime = RuntimeGuard::new();
    // SAFETY: Version getters return process-lifetime, NUL-terminated strings.
    let pairs = unsafe {
        [
            (ta::version_string(), sys::TA_GetVersionString()),
            (ta::version_major(), sys::TA_GetVersionMajor()),
            (ta::version_minor(), sys::TA_GetVersionMinor()),
            (ta::version_patch(), sys::TA_GetVersionPatch()),
            (ta::version_date(), sys::TA_GetVersionDate()),
            (ta::version_time(), sys::TA_GetVersionTime()),
            (ta::version_build(), sys::TA_GetVersionBuild()),
            (ta::version_extra(), sys::TA_GetVersionExtra()),
        ]
    };
    for (wrapper, native) in pairs {
        assert!(!native.is_null());
        // SAFETY: Each pointer above comes from a static native version getter.
        assert_eq!(wrapper, unsafe { CStr::from_ptr(native) }.to_str().unwrap());
    }
    for component in [
        ta::version_major(),
        ta::version_minor(),
        ta::version_patch(),
    ] {
        component
            .parse::<u32>()
            .expect("numeric semantic version component");
    }
    let prefix = format!(
        "{}.{}.{}",
        ta::version_major(),
        ta::version_minor(),
        ta::version_patch()
    );
    assert!(ta::version_string().starts_with(&prefix));
    assert!(!ta::version_date().is_empty());
    assert!(!ta::version_time().is_empty());
}

const UNSTABLE_IDS: &[ta::FuncUnstId] = &[
    ta::FuncUnstId::TA_FUNC_UNST_ADX,
    ta::FuncUnstId::TA_FUNC_UNST_UNUSED_1,
    ta::FuncUnstId::TA_FUNC_UNST_ATR,
    ta::FuncUnstId::TA_FUNC_UNST_CMO,
    ta::FuncUnstId::TA_FUNC_UNST_DX,
    ta::FuncUnstId::TA_FUNC_UNST_EMA,
    ta::FuncUnstId::TA_FUNC_UNST_HT_DCPERIOD,
    ta::FuncUnstId::TA_FUNC_UNST_HT_DCPHASE,
    ta::FuncUnstId::TA_FUNC_UNST_HT_PHASOR,
    ta::FuncUnstId::TA_FUNC_UNST_HT_SINE,
    ta::FuncUnstId::TA_FUNC_UNST_HT_TRENDLINE,
    ta::FuncUnstId::TA_FUNC_UNST_HT_TRENDMODE,
    ta::FuncUnstId::TA_FUNC_UNST_UNUSED_12,
    ta::FuncUnstId::TA_FUNC_UNST_KAMA,
    ta::FuncUnstId::TA_FUNC_UNST_MAMA,
    ta::FuncUnstId::TA_FUNC_UNST_UNUSED_15,
    ta::FuncUnstId::TA_FUNC_UNST_MINUS_DI,
    ta::FuncUnstId::TA_FUNC_UNST_MINUS_DM,
    ta::FuncUnstId::TA_FUNC_UNST_NATR,
    ta::FuncUnstId::TA_FUNC_UNST_PLUS_DI,
    ta::FuncUnstId::TA_FUNC_UNST_PLUS_DM,
    ta::FuncUnstId::TA_FUNC_UNST_RSI,
    ta::FuncUnstId::TA_FUNC_UNST_UNUSED_22,
    ta::FuncUnstId::TA_FUNC_UNST_T3,
    ta::FuncUnstId::TA_FUNC_UNST_RMA,
    ta::FuncUnstId::TA_FUNC_UNST_HA,
    ta::FuncUnstId::TA_FUNC_UNST_RVI,
    #[cfg(not(feature = "stable"))]
    ta::FuncUnstId::TA_FUNC_UNST_FRAMA,
    #[cfg(not(feature = "stable"))]
    ta::FuncUnstId::TA_FUNC_UNST_MCGD,
    #[cfg(not(feature = "stable"))]
    ta::FuncUnstId::TA_FUNC_UNST_VIDYA,
    #[cfg(not(feature = "stable"))]
    ta::FuncUnstId::TA_FUNC_UNST_STC,
    #[cfg(not(feature = "stable"))]
    ta::FuncUnstId::TA_FUNC_UNST_SWAK_GAUSS,
    #[cfg(not(feature = "stable"))]
    ta::FuncUnstId::TA_FUNC_UNST_SWAK_BUTTER,
    #[cfg(not(feature = "stable"))]
    ta::FuncUnstId::TA_FUNC_UNST_SWAK_HP,
    #[cfg(not(feature = "stable"))]
    ta::FuncUnstId::TA_FUNC_UNST_SWAK_2PHP,
    #[cfg(not(feature = "stable"))]
    ta::FuncUnstId::TA_FUNC_UNST_SWAK_BP,
    #[cfg(not(feature = "stable"))]
    ta::FuncUnstId::TA_FUNC_UNST_FISHER,
];

#[test]
fn unstable_periods_cover_every_id_and_all_broadcast() {
    let _runtime = RuntimeGuard::new();
    assert_eq!(UNSTABLE_IDS.len(), sys::TA_FUNC_UNST_COUNT);
    for (index, &id) in UNSTABLE_IDS.iter().enumerate() {
        let period = index as u32 + 1;
        ta::set_unstable_period(id, period).unwrap();
        assert_eq!(ta::get_unstable_period(id), period, "{id:?}");
    }
    for period in [0, 11, sys::TA_MAX_INDEX as u32] {
        ta::set_unstable_period(ta::FuncUnstId::TA_FUNC_UNST_ALL, period).unwrap();
        for &id in UNSTABLE_IDS {
            assert_eq!(ta::get_unstable_period(id), period);
        }
        assert_eq!(ta::get_unstable_period(ta::FuncUnstId::TA_FUNC_UNST_ALL), 0);
    }
    assert_eq!(
        ta::set_unstable_period(ta::FuncUnstId::TA_FUNC_UNST_ALL, u32::MAX),
        Err(ta::Error::BadParam)
    );
    for &id in UNSTABLE_IDS {
        assert_eq!(ta::get_unstable_period(id), sys::TA_MAX_INDEX as u32);
    }
}

#[test]
fn deprecated_compatibility_modes_preserve_default_behavior() {
    let _runtime = RuntimeGuard::new();
    // Upstream accepts both legacy modes but always uses default behavior.
    for mode in [Compatibility::Default, Compatibility::Metastock] {
        ta::set_compatibility(mode).unwrap();
        assert_eq!(ta::get_compatibility(), Compatibility::Default);
        assert_eq!(
            // SAFETY: The runtime is initialized and this getter has no pointer inputs.
            unsafe { sys::TA_GetCompatibility() },
            sys::TA_Compatibility::TA_COMPATIBILITY_DEFAULT
        );
    }
}

#[test]
fn candle_settings_cover_all_ranges_and_restore_targets() {
    let _runtime = RuntimeGuard::new();
    let settings = [
        Candle::BodyLong,
        Candle::BodyVeryLong,
        Candle::BodyShort,
        Candle::BodyDoji,
        Candle::ShadowLong,
        Candle::ShadowVeryLong,
        Candle::ShadowShort,
        Candle::ShadowVeryShort,
        Candle::Near,
        Candle::Far,
        Candle::Equal,
    ];
    for setting in settings {
        for range in [RangeType::RealBody, RangeType::HighLow, RangeType::Shadows] {
            ta::set_candle_settings(setting, range, 7, 0.75).unwrap();
            if setting == Candle::BodyDoji {
                // SAFETY: The runtime is initialized; this lookback takes no pointer inputs.
                assert_eq!(unsafe { sys::TA_CDLDOJI_Lookback() }, 7);
            }
            ta::restore_candle_default_settings(setting).unwrap();
            if setting == Candle::BodyDoji {
                // SAFETY: Restoring defaults makes the native doji lookback ten bars.
                assert_eq!(unsafe { sys::TA_CDLDOJI_Lookback() }, 10);
            }
        }
        for (period, factor) in [(-1, 1.0), (sys::TA_MAX_INDEX + 1, 1.0), (1, f64::NAN)] {
            assert_eq!(
                ta::set_candle_settings(setting, RangeType::RealBody, period, factor),
                Err(ta::Error::BadParam)
            );
        }
    }
    for range in [RangeType::RealBody, RangeType::HighLow, RangeType::Shadows] {
        assert_eq!(
            ta::set_candle_settings(Candle::AllCandleSettings, range, 7, 0.75),
            Err(ta::Error::BadParam)
        );
    }
    ta::restore_candle_default_settings(Candle::AllCandleSettings).unwrap();
}

#[test]
fn return_code_diagnostics_cover_all_native_variants() {
    let _runtime = RuntimeGuard::new();
    let codes = [
        sys::TA_RetCode::TA_SUCCESS,
        sys::TA_RetCode::TA_LIB_NOT_INITIALIZE,
        sys::TA_RetCode::TA_BAD_PARAM,
        sys::TA_RetCode::TA_ALLOC_ERR,
        sys::TA_RetCode::TA_GROUP_NOT_FOUND,
        sys::TA_RetCode::TA_FUNC_NOT_FOUND,
        sys::TA_RetCode::TA_INVALID_HANDLE,
        sys::TA_RetCode::TA_INVALID_PARAM_HOLDER,
        sys::TA_RetCode::TA_INVALID_PARAM_HOLDER_TYPE,
        sys::TA_RetCode::TA_INVALID_PARAM_FUNCTION,
        sys::TA_RetCode::TA_INPUT_NOT_ALL_INITIALIZE,
        sys::TA_RetCode::TA_OUTPUT_NOT_ALL_INITIALIZE,
        sys::TA_RetCode::TA_OUT_OF_RANGE_START_INDEX,
        sys::TA_RetCode::TA_OUT_OF_RANGE_END_INDEX,
        sys::TA_RetCode::TA_INVALID_LIST_TYPE,
        sys::TA_RetCode::TA_BAD_OBJECT,
        sys::TA_RetCode::TA_NOT_SUPPORTED,
        sys::TA_RetCode::TA_INSUFFICIENT_HISTORY,
        sys::TA_RetCode::TA_INTERNAL_ERROR,
        sys::TA_RetCode::TA_UNKNOWN_ERR,
    ];
    let mut names = BTreeSet::new();
    for code in codes {
        let info = ta::ret_code_info(code);
        assert_eq!(info.enum_str, format!("{code:?}"));
        assert!(!info.info_str.is_empty());
        assert!(names.insert(info.enum_str.clone()));
        let mut native = sys::TA_RetCodeInfo {
            enumStr: ptr::null(),
            infoStr: ptr::null(),
        };
        // SAFETY: Native code writes static string pointers to this valid output structure.
        unsafe {
            sys::TA_SetRetCodeInfo(code, &mut native);
            assert!(!native.enumStr.is_null() && !native.infoStr.is_null());
            assert_eq!(
                info.enum_str,
                CStr::from_ptr(native.enumStr).to_str().unwrap()
            );
            assert_eq!(
                info.info_str,
                CStr::from_ptr(native.infoStr).to_str().unwrap()
            );
        }
        assert_eq!(ta::check_ret_code(code).is_ok(), code == TA_SUCCESS);
    }
}

struct StringTable {
    raw: *mut sys::TA_StringTable,
    group: bool,
}
impl StringTable {
    fn names(&self) -> Vec<String> {
        // SAFETY: The successful allocation owns the table and all its strings until Drop.
        unsafe {
            let table = &*self.raw;
            assert!(table.size > 0);
            assert!(!table.string.is_null());
            std::slice::from_raw_parts(table.string, table.size as usize)
                .iter()
                .map(|&name| {
                    assert!(!name.is_null());
                    CStr::from_ptr(name).to_str().unwrap().to_owned()
                })
                .collect()
        }
    }
}
impl Drop for StringTable {
    fn drop(&mut self) {
        // SAFETY: The table has exactly one owner and is freed by its matching allocator pair.
        let code = unsafe {
            if self.group {
                sys::TA_GroupTableFree(self.raw)
            } else {
                sys::TA_FuncTableFree(self.raw)
            }
        };
        assert_eq!(code, TA_SUCCESS);
    }
}
struct ParamHolder(*mut sys::TA_ParamHolder);
impl Drop for ParamHolder {
    fn drop(&mut self) {
        // SAFETY: This holder was successfully allocated and has not been freed.
        assert_eq!(unsafe { sys::TA_ParamHolderFree(self.0) }, TA_SUCCESS);
    }
}

#[test]
fn abstract_abi_allocates_and_inspects_all_201_functions() {
    let _runtime = RuntimeGuard::new();
    let mut raw_groups = ptr::null_mut();
    assert_eq!(
        // SAFETY: The runtime is initialized and the output points to writable pointer storage.
        unsafe { sys::TA_GroupTableAlloc(&mut raw_groups) },
        TA_SUCCESS
    );
    assert!(!raw_groups.is_null());
    let groups = StringTable {
        raw: raw_groups,
        group: true,
    };
    let mut functions = BTreeSet::new();
    let mut group_names = BTreeSet::new();
    for group in groups.names() {
        assert!(group_names.insert(group.clone()));
        let c_group = CString::new(group.as_str()).unwrap();
        let mut raw_functions = ptr::null_mut();
        assert_eq!(
            // SAFETY: The group string is NUL-terminated and output storage is writable.
            unsafe { sys::TA_FuncTableAlloc(c_group.as_ptr(), &mut raw_functions) },
            TA_SUCCESS
        );
        assert!(!raw_functions.is_null());
        let table = StringTable {
            raw: raw_functions,
            group: false,
        };
        for name in table.names() {
            assert!(functions.insert(name.clone()), "duplicate function {name}");
            let c_name = CString::new(name.as_str()).unwrap();
            let mut handle = ptr::null();
            let mut info = ptr::null();
            let mut holder = ptr::null_mut();
            // SAFETY: Function names come from the live native table; all output pointers
            // reference writable storage. Metadata remains valid until runtime shutdown.
            unsafe {
                assert_eq!(
                    sys::TA_GetFuncHandle(c_name.as_ptr(), &mut handle),
                    TA_SUCCESS
                );
                assert!(!handle.is_null());
                assert_eq!(sys::TA_GetFuncInfo(handle, &mut info), TA_SUCCESS);
                assert!(!info.is_null());
                let info = &*info;
                assert_eq!(info.handle, handle);
                assert!(!info.name.is_null() && !info.group.is_null());
                assert_eq!(CStr::from_ptr(info.name).to_str().unwrap(), name);
                assert_eq!(CStr::from_ptr(info.group).to_str().unwrap(), group);
                assert!(info.nbInput > 0 && info.nbOutput > 0);
                assert_eq!(sys::TA_ParamHolderAlloc(handle, &mut holder), TA_SUCCESS);
                assert!(!holder.is_null());
                let _holder = ParamHolder(holder);
                assert!(!(*holder).hiddenData.is_null());
                for index in 0..info.nbInput {
                    let mut param = ptr::null();
                    assert_eq!(
                        sys::TA_GetInputParameterInfo(handle, index, &mut param),
                        TA_SUCCESS
                    );
                    assert!(!param.is_null() && !(*param).paramName.is_null());
                    assert!(!CStr::from_ptr((*param).paramName).to_bytes().is_empty());
                }
                for index in 0..info.nbOptInput {
                    let mut param = ptr::null();
                    assert_eq!(
                        sys::TA_GetOptInputParameterInfo(handle, index, &mut param),
                        TA_SUCCESS
                    );
                    assert!(!param.is_null() && !(*param).paramName.is_null());
                    assert!(!CStr::from_ptr((*param).paramName).to_bytes().is_empty());
                }
                for index in 0..info.nbOutput {
                    let mut param = ptr::null();
                    assert_eq!(
                        sys::TA_GetOutputParameterInfo(handle, index, &mut param),
                        TA_SUCCESS
                    );
                    assert!(!param.is_null() && !(*param).paramName.is_null());
                    assert!(!CStr::from_ptr((*param).paramName).to_bytes().is_empty());
                }
            }
        }
    }
    let expected_funcs = if cfg!(feature = "stable") { 201 } else { 237 };
    assert_eq!(functions.len(), expected_funcs);
    assert_eq!(functions.len(), sys::GENERATED_STREAM_TYPE_COUNT);
    assert!(functions.contains("SMA") && functions.contains("CDLDOJI"));
}

#[test]
fn non_finite_batch_values_succeed_but_stream_updates_reject_them() {
    let _runtime = RuntimeGuard::new();
    let batch = [1.0, 2.0, f64::NAN, 4.0, 5.0];
    let (values, begin) = ta::sma(&batch, Some(3)).expect("batch SMA accepts NaN");
    assert_eq!(begin, 2);
    assert!(values.iter().any(|value| value.is_nan()));

    let infinite = [1.0, f64::INFINITY, 3.0, 4.0];
    let (values, _) = ta::sma(&infinite, Some(3)).expect("batch SMA accepts infinity");
    assert!(values.iter().any(|value| !value.is_finite()));

    let (mut stream, _) = ta::SmaStream::open(&[1.0, 2.0, 3.0, 4.0], Some(3)).expect("SMA stream");
    assert_eq!(stream.update(f64::NAN), Err(ta::Error::BadParam));
    assert_eq!(stream.peek(f64::INFINITY), Err(ta::Error::BadParam));
    assert_eq!(stream.update(f64::INFINITY), Err(ta::Error::BadParam));
    assert!(stream.value().is_ok());
}

#[test]
fn stream_advance_extends_out_range_without_changing_value() {
    let _runtime = RuntimeGuard::new();
    let (mut stream, _) = ta::SmaStream::open(&[1.0, 2.0, 3.0, 4.0], Some(3)).expect("SMA stream");
    let before_range = stream.out_range().expect("out range");
    let before_value = stream.value().expect("value");
    stream.advance().expect("advance");
    let after_range = stream.out_range().expect("out range");
    assert_eq!(after_range.begin, before_range.begin);
    assert_eq!(after_range.count, before_range.count + 1);
    assert_eq!(stream.value().expect("value"), before_value);
}
