//! Reflective Abstract API (`ta_abstract.h`) for dynamic function discovery and invocation.
//! Tables are owned through RAII and the parameter holder validates buffer lengths.

use core::{
    ffi::{CStr, c_char, c_void},
    marker::PhantomData,
    ptr,
};
use std::ffi::CString;

use ta_lib_sys as sys;

use crate::{
    error::{Error, check_ret_code},
    types::OutRange,
};

/// Copy a NUL-terminated string owned by TA-Lib, treating a null pointer as empty.
fn c_string(pointer: *const c_char) -> String {
    if pointer.is_null() {
        return String::new();
    }
    // SAFETY: every string reachable through the Abstract API is NUL-terminated and remains
    // valid for the lifetime of the loaded library.
    unsafe { CStr::from_ptr(pointer).to_string_lossy().into_owned() }
}

type TableFree = unsafe extern "C" fn(*mut sys::TA_StringTable) -> sys::TA_RetCode;

/// RAII owner for a table allocated by `TA_GroupTableAlloc` or `TA_FuncTableAlloc`.
struct StringTable {
    raw: *mut sys::TA_StringTable,
    free: TableFree,
}

impl StringTable {
    /// Snapshot the entries before the table is freed.
    fn entries(&self) -> Vec<String> {
        // SAFETY: `raw` was returned by a successful `*TableAlloc` and is owned by `self`.
        let table = unsafe { &*self.raw };
        if table.size == 0 || table.string.is_null() {
            return Vec::new();
        }
        // SAFETY: the allocator writes `size` valid string pointers into `string`.
        let entries = unsafe { core::slice::from_raw_parts(table.string, table.size as usize) };
        entries.iter().copied().map(c_string).collect()
    }
}

impl Drop for StringTable {
    fn drop(&mut self) {
        // SAFETY: `raw` was allocated by the matching allocator in `free`, and this is the only
        // owner that frees it.
        unsafe { (self.free)(self.raw) };
    }
}

/// Return every TA-Lib function group name (`TA_GroupTableAlloc` / `TA_GroupTableFree`).
/// # Errors
/// Returns an [`Error`] if TA-Lib cannot allocate or fill the group table.
pub fn groups() -> Result<Vec<String>, Error> {
    let mut raw: *mut sys::TA_StringTable = ptr::null_mut();
    // SAFETY: `raw` is a valid out-pointer and receives an owned table on success.
    let rc = unsafe { sys::TA_GroupTableAlloc(&raw mut raw) };
    check_ret_code(rc)?;
    if raw.is_null() {
        return Err(Error::BadParam);
    }
    Ok(StringTable {
        raw,
        free: sys::TA_GroupTableFree,
    }
    .entries())
}

/// Return every function name in `group` (`TA_FuncTableAlloc` / `TA_FuncTableFree`).
/// # Errors
/// Returns an [`Error`] if `group` contains an interior NUL byte, is unknown, or the table
/// cannot be allocated.
pub fn functions_in_group(group: &str) -> Result<Vec<String>, Error> {
    let group = CString::new(group).map_err(|_| Error::InteriorNul)?;
    let mut raw: *mut sys::TA_StringTable = ptr::null_mut();
    // SAFETY: `group` is NUL-terminated and `raw` receives an owned table on success.
    let rc = unsafe { sys::TA_FuncTableAlloc(group.as_ptr(), &raw mut raw) };
    check_ret_code(rc)?;
    if raw.is_null() {
        return Err(Error::BadParam);
    }
    Ok(StringTable {
        raw,
        free: sys::TA_FuncTableFree,
    }
    .entries())
}

/// Return every function name across all groups, concatenating the tables in [`groups`] order.
/// # Errors
/// Returns an [`Error`] if any group table cannot be allocated.
pub fn all_functions() -> Result<Vec<String>, Error> {
    let mut functions = Vec::new();
    for group in groups()? {
        functions.extend(functions_in_group(&group)?);
    }
    Ok(functions)
}

/// Return the static XML description of every function (`TA_FunctionDescriptionXML`).
#[must_use]
pub fn function_description_xml() -> &'static str {
    // SAFETY: `TA_FunctionDescriptionXML` returns a static NUL-terminated string.
    let pointer = unsafe { sys::TA_FunctionDescriptionXML() };
    if pointer.is_null() {
        return "";
    }
    // SAFETY: the pointer is a static NUL-terminated string owned by the library.
    unsafe { CStr::from_ptr(pointer).to_str().unwrap_or("") }
}

/// Opaque handle to a TA-Lib function definition (`TA_FuncHandle`).
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct FuncHandle {
    raw: *const sys::TA_FuncHandle,
}

impl FuncHandle {
    /// Look up a function by name (`TA_GetFuncHandle`). Matching is case-insensitive.
    /// # Errors
    /// Returns an [`Error`] if `name` contains an interior NUL byte or names no known function.
    pub fn get(name: &str) -> Result<Self, Error> {
        let name = CString::new(name).map_err(|_| Error::InteriorNul)?;
        let mut raw: *const sys::TA_FuncHandle = ptr::null();
        // SAFETY: `name` is NUL-terminated; on success TA-Lib writes a handle backed by static
        // function-definition storage.
        let rc = unsafe { sys::TA_GetFuncHandle(name.as_ptr(), &raw mut raw) };
        check_ret_code(rc)?;
        if raw.is_null() {
            return Err(Error::BadParam);
        }
        Ok(Self { raw })
    }

    /// Snapshot the metadata for this function (`TA_GetFuncInfo`).
    /// # Errors
    /// Returns an [`Error`] if TA-Lib rejects the handle.
    pub fn info(&self) -> Result<FuncInfo, Error> {
        let mut info: *const sys::TA_FuncInfo = ptr::null();
        // SAFETY: `self.raw` is a valid handle; TA-Lib writes a pointer into static metadata.
        let rc = unsafe { sys::TA_GetFuncInfo(self.raw, &raw mut info) };
        check_ret_code(rc)?;
        FuncInfo::from_raw(info).ok_or(Error::BadParam)
    }

    /// Describe input parameter `index` (`TA_GetInputParameterInfo`).
    /// # Errors
    /// Returns an [`Error`] if `index` is out of range for this function.
    pub fn input_parameter_info(&self, index: u32) -> Result<InputParameterInfo, Error> {
        let info: *const sys::TA_InputParameterInfo =
            self.raw_parameter_info(index, sys::TA_GetInputParameterInfo)?;
        // SAFETY: successful lookup yields a pointer into static metadata.
        let info = unsafe { info.as_ref() }.ok_or(Error::BadParam)?;
        Ok(InputParameterInfo {
            parameter_type: input_parameter_type(info.type_),
            name: c_string(info.paramName),
            flags: info.flags,
        })
    }

    /// Describe optional input parameter `index` (`TA_GetOptInputParameterInfo`).
    /// # Errors
    /// Returns an [`Error`] if `index` is out of range for this function.
    pub fn opt_input_parameter_info(&self, index: u32) -> Result<OptInputParameterInfo, Error> {
        let info: *const sys::TA_OptInputParameterInfo =
            self.raw_parameter_info(index, sys::TA_GetOptInputParameterInfo)?;
        // SAFETY: successful lookup yields a pointer into static metadata.
        let info = unsafe { info.as_ref() }.ok_or(Error::BadParam)?;
        Ok(OptInputParameterInfo {
            parameter_type: opt_input_parameter_type(info.type_),
            name: c_string(info.paramName),
            flags: info.flags,
            display_name: c_string(info.displayName),
            default_value: info.defaultValue,
            hint: c_string(info.hint),
            help_file: c_string(info.helpFile),
            data_set: opt_input_data_set(info.type_, info.dataSet),
        })
    }

    /// Describe output parameter `index` (`TA_GetOutputParameterInfo`).
    /// # Errors
    /// Returns an [`Error`] if `index` is out of range for this function.
    pub fn output_parameter_info(&self, index: u32) -> Result<OutputParameterInfo, Error> {
        let info: *const sys::TA_OutputParameterInfo =
            self.raw_parameter_info(index, sys::TA_GetOutputParameterInfo)?;
        // SAFETY: successful lookup yields a pointer into static metadata.
        let info = unsafe { info.as_ref() }.ok_or(Error::BadParam)?;
        Ok(OutputParameterInfo {
            parameter_type: output_parameter_type(info.type_),
            name: c_string(info.paramName),
            flags: info.flags,
        })
    }

    /// Run one of the three parameter-info getters with a shared out-pointer protocol.
    fn raw_parameter_info<T>(
        &self,
        index: u32,
        getter: unsafe extern "C" fn(
            *const sys::TA_FuncHandle,
            core::ffi::c_uint,
            *mut *const T,
        ) -> sys::TA_RetCode,
    ) -> Result<*const T, Error> {
        let mut info: *const T = ptr::null();
        // SAFETY: `self.raw` is a valid handle, `getter` is one of the matching library
        // functions, and `info` is a valid out-pointer for its `T`.
        let rc = unsafe { getter(self.raw, index, &raw mut info) };
        check_ret_code(rc)?;
        if info.is_null() {
            return Err(Error::BadParam);
        }
        Ok(info)
    }
}

/// Owned snapshot of a `TA_FuncInfo` record.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FuncInfo {
    pub name: String,
    pub group: String,
    pub hint: String,
    pub flags: sys::TA_FuncFlags,
    pub input_count: usize,
    pub opt_input_count: usize,
    pub output_count: usize,
    pub handle: FuncHandle,
}

impl FuncInfo {
    /// Copy a C `TA_FuncInfo` into owned Rust data.
    fn from_raw(info: *const sys::TA_FuncInfo) -> Option<Self> {
        // SAFETY: callers pass a non-null pointer into TA-Lib static metadata, or null.
        let info = unsafe { info.as_ref() }?;
        if info.handle.is_null() {
            return None;
        }
        Some(Self {
            name: c_string(info.name),
            group: c_string(info.group),
            hint: c_string(info.hint),
            flags: info.flags,
            input_count: info.nbInput as usize,
            opt_input_count: info.nbOptInput as usize,
            output_count: info.nbOutput as usize,
            handle: FuncHandle { raw: info.handle },
        })
    }
}

struct Visitor<'a> {
    callback: &'a mut dyn FnMut(&FuncInfo),
}

unsafe extern "C" fn visit(info: *const sys::TA_FuncInfo, opaque: *mut c_void) {
    // SAFETY: `for_each_function` passes a pointer to a live `Visitor` for the whole traversal,
    // and `info` is a valid static record for the duration of the callback.
    let visitor = unsafe { &mut *opaque.cast::<Visitor<'_>>() };
    if let Some(info) = FuncInfo::from_raw(info) {
        (visitor.callback)(&info);
    }
}

/// Visit every registered function (`TA_ForEachFunc`).
/// # Errors
/// Returns an [`Error`] if TA-Lib reports an inconsistent function table.
pub fn for_each_function(callback: &mut dyn FnMut(&FuncInfo)) -> Result<(), Error> {
    let mut visitor = Visitor { callback };
    // SAFETY: `visit` matches `TA_CallForEachFunc`, and `visitor` outlives the synchronous
    // traversal.
    let rc = unsafe { sys::TA_ForEachFunc(Some(visit), (&raw mut visitor).cast::<c_void>()) };
    check_ret_code(rc)
}

/// Required input parameter kind (`TA_InputParameterType`).
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum InputParameterType {
    Price,
    Real,
    Integer,
}

/// Owned snapshot of a `TA_InputParameterInfo` record.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InputParameterInfo {
    pub parameter_type: InputParameterType,
    pub name: String,
    pub flags: sys::TA_InputFlags,
}

/// Optional input parameter kind (`TA_OptInputParameterType`).
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum OptInputParameterType {
    RealRange,
    RealList,
    IntegerRange,
    IntegerList,
}

/// Continuous range metadata for a real-valued optional input.
#[derive(Debug, Copy, Clone, PartialEq, Default)]
pub struct RealRange {
    pub min: f64,
    pub max: f64,
    pub precision: i32,
    pub suggested_start: f64,
    pub suggested_end: f64,
    pub suggested_increment: f64,
}

/// Range metadata for an integer-valued optional input.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Default)]
pub struct IntegerRange {
    pub min: i32,
    pub max: i32,
    pub suggested_start: i32,
    pub suggested_end: i32,
    pub suggested_increment: i32,
}

/// Named real value from a `TA_RealList`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct NamedReal {
    pub name: String,
    pub value: f64,
}

/// Named integer value from a `TA_IntegerList`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct NamedInteger {
    pub name: String,
    pub value: i32,
}

/// Domain metadata carried by an optional input parameter.
#[derive(Debug, Clone, PartialEq)]
pub enum OptInputDataSet {
    RealRange(RealRange),
    IntegerRange(IntegerRange),
    RealList(Vec<NamedReal>),
    IntegerList(Vec<NamedInteger>),
}

/// Owned snapshot of a `TA_OptInputParameterInfo` record.
#[derive(Debug, Clone, PartialEq)]
pub struct OptInputParameterInfo {
    pub parameter_type: OptInputParameterType,
    pub name: String,
    pub flags: sys::TA_OptInputFlags,
    pub display_name: String,
    pub default_value: f64,
    pub hint: String,
    pub help_file: String,
    pub data_set: OptInputDataSet,
}

/// Output parameter kind (`TA_OutputParameterType`).
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum OutputParameterType {
    Real,
    Integer,
}

/// Owned snapshot of a `TA_OutputParameterInfo` record.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OutputParameterInfo {
    pub parameter_type: OutputParameterType,
    pub name: String,
    pub flags: sys::TA_OutputFlags,
}

fn input_parameter_type(value: sys::TA_InputParameterType) -> InputParameterType {
    match value {
        sys::TA_InputParameterType::TA_Input_Price => InputParameterType::Price,
        sys::TA_InputParameterType::TA_Input_Real => InputParameterType::Real,
        sys::TA_InputParameterType::TA_Input_Integer => InputParameterType::Integer,
    }
}

fn opt_input_parameter_type(value: sys::TA_OptInputParameterType) -> OptInputParameterType {
    match value {
        sys::TA_OptInputParameterType::TA_OptInput_RealRange => OptInputParameterType::RealRange,
        sys::TA_OptInputParameterType::TA_OptInput_RealList => OptInputParameterType::RealList,
        sys::TA_OptInputParameterType::TA_OptInput_IntegerRange => {
            OptInputParameterType::IntegerRange
        }
        sys::TA_OptInputParameterType::TA_OptInput_IntegerList => {
            OptInputParameterType::IntegerList
        }
    }
}

fn output_parameter_type(value: sys::TA_OutputParameterType) -> OutputParameterType {
    match value {
        sys::TA_OutputParameterType::TA_Output_Real => OutputParameterType::Real,
        sys::TA_OutputParameterType::TA_Output_Integer => OutputParameterType::Integer,
    }
}

/// Interpret the `dataSet` union according to the optional input kind.
fn opt_input_data_set(
    parameter_type: sys::TA_OptInputParameterType,
    data_set: *const c_void,
) -> OptInputDataSet {
    match parameter_type {
        sys::TA_OptInputParameterType::TA_OptInput_RealRange => {
            // SAFETY: TA-Lib stores a `TA_RealRange` for this kind, or null.
            let range = unsafe { data_set.cast::<sys::TA_RealRange>().as_ref() };
            OptInputDataSet::RealRange(range.map_or_else(RealRange::default, |range| RealRange {
                min: range.min,
                max: range.max,
                precision: range.precision,
                suggested_start: range.suggested_start,
                suggested_end: range.suggested_end,
                suggested_increment: range.suggested_increment,
            }))
        }
        sys::TA_OptInputParameterType::TA_OptInput_IntegerRange => {
            // SAFETY: TA-Lib stores a `TA_IntegerRange` for this kind, or null.
            let range = unsafe { data_set.cast::<sys::TA_IntegerRange>().as_ref() };
            OptInputDataSet::IntegerRange(range.map_or_else(IntegerRange::default, |range| {
                IntegerRange {
                    min: range.min,
                    max: range.max,
                    suggested_start: range.suggested_start,
                    suggested_end: range.suggested_end,
                    suggested_increment: range.suggested_increment,
                }
            }))
        }
        sys::TA_OptInputParameterType::TA_OptInput_RealList => {
            OptInputDataSet::RealList(real_list(data_set.cast::<sys::TA_RealList>()))
        }
        sys::TA_OptInputParameterType::TA_OptInput_IntegerList => {
            OptInputDataSet::IntegerList(integer_list(data_set.cast::<sys::TA_IntegerList>()))
        }
    }
}

/// Copy a `TA_RealList` into owned pairs.
fn real_list(list: *const sys::TA_RealList) -> Vec<NamedReal> {
    // SAFETY: TA-Lib stores a `TA_RealList` for the real-list kind, or null.
    let Some(list) = (unsafe { list.as_ref() }) else {
        return Vec::new();
    };
    if list.data.is_null() || list.nbElement == 0 {
        return Vec::new();
    }
    // SAFETY: the list owns `nbElement` contiguous pairs.
    let pairs = unsafe { core::slice::from_raw_parts(list.data, list.nbElement as usize) };
    pairs
        .iter()
        .map(|pair| NamedReal {
            name: c_string(pair.string),
            value: pair.value,
        })
        .collect()
}

/// Copy a `TA_IntegerList` into owned pairs.
fn integer_list(list: *const sys::TA_IntegerList) -> Vec<NamedInteger> {
    // SAFETY: TA-Lib stores a `TA_IntegerList` for the integer-list kind, or null.
    let Some(list) = (unsafe { list.as_ref() }) else {
        return Vec::new();
    };
    if list.data.is_null() || list.nbElement == 0 {
        return Vec::new();
    }
    // SAFETY: the list owns `nbElement` contiguous pairs.
    let pairs = unsafe { core::slice::from_raw_parts(list.data, list.nbElement as usize) };
    pairs
        .iter()
        .map(|pair| NamedInteger {
            name: c_string(pair.string),
            value: pair.value,
        })
        .collect()
}

fn slice_ptr(slice: Option<&[f64]>) -> *const f64 {
    slice.map_or(ptr::null(), <[f64]>::as_ptr)
}

/// Safe owner of a `TA_ParamHolder` that validates buffer lengths before `TA_CallFunc`.
/// Registered slices stay borrowed until the holder is dropped.
pub struct ParamHolder<'a> {
    raw: *mut sys::TA_ParamHolder,
    input_lengths: Vec<Option<usize>>,
    output_capacities: Vec<Option<usize>>,
    _borrow: PhantomData<&'a mut ()>,
}

impl<'a> ParamHolder<'a> {
    /// Allocate a holder for `handle` (`TA_ParamHolderAlloc`).
    /// # Errors
    /// Returns an [`Error`] if TA-Lib rejects the handle or cannot allocate the holder.
    pub fn new(handle: &FuncHandle) -> Result<Self, Error> {
        let info = handle.info()?;
        let mut raw: *mut sys::TA_ParamHolder = ptr::null_mut();
        // SAFETY: `handle.raw` is valid and `raw` receives the owned holder on success.
        let rc = unsafe { sys::TA_ParamHolderAlloc(handle.raw, &raw mut raw) };
        check_ret_code(rc)?;
        if raw.is_null() {
            return Err(Error::BadParam);
        }
        Ok(Self {
            raw,
            input_lengths: vec![None; info.input_count],
            output_capacities: vec![None; info.output_count],
            _borrow: PhantomData,
        })
    }

    /// Bind a real-valued input series (`TA_SetInputParamRealPtr`).
    /// # Errors
    /// Returns an [`Error`] if `index` does not name a real input of this function.
    pub fn set_input_real(&mut self, index: u32, values: &'a [f64]) -> Result<(), Error> {
        // SAFETY: `values` is a valid slice that outlives the holder.
        let rc = unsafe { sys::TA_SetInputParamRealPtr(self.raw, index, values.as_ptr()) };
        check_ret_code(rc)?;
        if let Some(length) = self.input_lengths.get_mut(index as usize) {
            *length = Some(values.len());
        }
        Ok(())
    }

    /// Bind an integer-valued input series (`TA_SetInputParamIntegerPtr`).
    /// # Errors
    /// Returns an [`Error`] if `index` does not name an integer input of this function.
    pub fn set_input_integer(&mut self, index: u32, values: &'a [i32]) -> Result<(), Error> {
        // SAFETY: `values` is a valid slice that outlives the holder.
        let rc = unsafe { sys::TA_SetInputParamIntegerPtr(self.raw, index, values.as_ptr()) };
        check_ret_code(rc)?;
        if let Some(length) = self.input_lengths.get_mut(index as usize) {
            *length = Some(values.len());
        }
        Ok(())
    }

    /// Bind the price series consumed by a `TA_Input_Price` parameter.
    /// TA-Lib requires only the components named by the input flags; unused components may be
    /// `None`. The recorded input length is the shortest supplied series.
    /// # Errors
    /// Returns an [`Error`] if a required component is missing or `index` is invalid.
    #[allow(clippy::too_many_arguments)]
    pub fn set_input_price(
        &mut self,
        index: u32,
        open: Option<&'a [f64]>,
        high: Option<&'a [f64]>,
        low: Option<&'a [f64]>,
        close: Option<&'a [f64]>,
        volume: Option<&'a [f64]>,
        open_interest: Option<&'a [f64]>,
    ) -> Result<(), Error> {
        let length = [open, high, low, close, volume, open_interest]
            .into_iter()
            .flatten()
            .map(<[f64]>::len)
            .min();
        // SAFETY: every supplied pointer is null or a valid slice that outlives the holder.
        let rc = unsafe {
            sys::TA_SetInputParamPricePtr(
                self.raw,
                index,
                slice_ptr(open),
                slice_ptr(high),
                slice_ptr(low),
                slice_ptr(close),
                slice_ptr(volume),
                slice_ptr(open_interest),
            )
        };
        check_ret_code(rc)?;
        if let Some(slot) = self.input_lengths.get_mut(index as usize) {
            *slot = length;
        }
        Ok(())
    }

    /// Set an integer optional input (`TA_SetOptInputParamInteger`).
    /// # Errors
    /// Returns an [`Error`] if `index` does not name an integer optional input.
    pub fn set_opt_input_integer(&mut self, index: u32, value: i32) -> Result<(), Error> {
        // SAFETY: `self.raw` is a live holder; TA-Lib validates the index and kind.
        let rc = unsafe { sys::TA_SetOptInputParamInteger(self.raw, index, value) };
        check_ret_code(rc)
    }

    /// Set a real optional input (`TA_SetOptInputParamReal`).
    /// # Errors
    /// Returns an [`Error`] if `index` does not name a real optional input.
    pub fn set_opt_input_real(&mut self, index: u32, value: f64) -> Result<(), Error> {
        // SAFETY: `self.raw` is a live holder; TA-Lib validates the index and kind.
        let rc = unsafe { sys::TA_SetOptInputParamReal(self.raw, index, value) };
        check_ret_code(rc)
    }

    /// Bind a real-valued output buffer (`TA_SetOutputParamRealPtr`).
    /// # Errors
    /// Returns an [`Error`] if `index` does not name a real output of this function.
    pub fn set_output_real(&mut self, index: u32, values: &'a mut [f64]) -> Result<(), Error> {
        let capacity = values.len();
        // SAFETY: `values` is an exclusive slice that outlives the holder.
        let rc = unsafe { sys::TA_SetOutputParamRealPtr(self.raw, index, values.as_mut_ptr()) };
        check_ret_code(rc)?;
        if let Some(slot) = self.output_capacities.get_mut(index as usize) {
            *slot = Some(capacity);
        }
        Ok(())
    }

    /// Bind an integer-valued output buffer (`TA_SetOutputParamIntegerPtr`).
    /// # Errors
    /// Returns an [`Error`] if `index` does not name an integer output of this function.
    pub fn set_output_integer(&mut self, index: u32, values: &'a mut [i32]) -> Result<(), Error> {
        let capacity = values.len();
        // SAFETY: `values` is an exclusive slice that outlives the holder.
        let rc = unsafe { sys::TA_SetOutputParamIntegerPtr(self.raw, index, values.as_mut_ptr()) };
        check_ret_code(rc)?;
        if let Some(slot) = self.output_capacities.get_mut(index as usize) {
            *slot = Some(capacity);
        }
        Ok(())
    }

    /// Return the warmup length implied by the configured optional inputs (`TA_GetLookback`).
    /// # Errors
    /// Returns an [`Error`] if TA-Lib rejects the holder or configured parameters.
    pub fn lookback(&self) -> Result<usize, Error> {
        let mut lookback: i32 = 0;
        // SAFETY: `self.raw` is a live holder and `lookback` is a valid out-pointer.
        let rc = unsafe { sys::TA_GetLookback(self.raw, &raw mut lookback) };
        check_ret_code(rc)?;
        usize::try_from(lookback).map_err(|_| Error::BadParam)
    }

    /// Return the display shift for output `index` (`TA_GetDisplayShift`).
    /// # Errors
    /// Returns an [`Error`] if the function provides no display shift for `index`.
    pub fn display_shift(&self, index: u32) -> Result<i32, Error> {
        let mut shift: i32 = 0;
        // SAFETY: `self.raw` is a live holder and `shift` is a valid out-pointer.
        let rc = unsafe { sys::TA_GetDisplayShift(self.raw, index, &raw mut shift) };
        check_ret_code(rc)?;
        if shift == sys::TA_INTEGER_DEFAULT {
            return Err(Error::BadParam);
        }
        Ok(shift)
    }

    /// Run the configured function over the inclusive `start..=end` range (`TA_CallFunc`).
    /// Inputs must cover `end`; outputs must hold `end - start + 1` elements.
    /// # Errors
    /// Returns an [`Error`] if a buffer is missing or too short, or TA-Lib rejects the parameters.
    pub fn call(&self, start: usize, end: usize) -> Result<OutRange, Error> {
        let span = end
            .checked_sub(start)
            .and_then(|d| d.checked_add(1))
            .ok_or(Error::BadParam)?;
        if self
            .input_lengths
            .iter()
            .any(|l| l.is_none_or(|l| l <= end))
        {
            return Err(Error::BadParam);
        }
        for capacity in &self.output_capacities {
            match *capacity {
                Some(capacity) if capacity >= span => {}
                actual => {
                    return Err(Error::OutputBufferTooSmall {
                        required: span,
                        actual: actual.unwrap_or(0),
                    });
                }
            }
        }
        let start = i32::try_from(start).map_err(|_| Error::BadParam)?;
        let end = i32::try_from(end).map_err(|_| Error::BadParam)?;
        let (mut begin, mut count) = (0_i32, 0_i32);
        // SAFETY: the holder is live, all buffers were bound and length-checked above, and both
        // out-pointers target valid stack integers.
        let rc = unsafe { sys::TA_CallFunc(self.raw, start, end, &raw mut begin, &raw mut count) };
        check_ret_code(rc)?;
        let begin = usize::try_from(begin).map_err(|_| Error::BadParam)?;
        let count = usize::try_from(count).map_err(|_| Error::BadParam)?;
        Ok(OutRange { begin, count })
    }
}

impl Drop for ParamHolder<'_> {
    fn drop(&mut self) {
        // SAFETY: `raw` was allocated by `TA_ParamHolderAlloc` and this is its only owner. The
        // holder outlives every slice registered with it because those borrows share its `'a`.
        unsafe { sys::TA_ParamHolderFree(self.raw) };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The reflective tables expose all ten groups, all 237 functions, and the XML payload.
    #[test]
    fn catalog_exposes_all_groups_and_functions() {
        let groups = groups().expect("group table");
        assert_eq!(groups.len(), 10);
        assert!(groups.iter().any(|g| g == "Overlap Studies"));
        assert!(groups.iter().any(|g| g == "Momentum Indicators"));
        let mut functions = all_functions().expect("function catalog");
        assert_eq!(functions.len(), 237);
        assert!(functions.iter().any(|name| name == "SMA"));
        let overlap = functions_in_group("Overlap Studies").expect("overlap");
        assert!(overlap.iter().any(|name| name == "SMA"));
        let mut visited = Vec::new();
        for_each_function(&mut |info| visited.push(info.name.clone())).expect("traversal");
        visited.sort();
        functions.sort();
        assert_eq!(visited, functions);
        let xml = function_description_xml();
        assert!(xml.starts_with("<?xml") && xml.contains("<FinancialFunctions>"));
    }

    fn sma_holder<'a>(input: &'a [f64], output: &'a mut [f64]) -> ParamHolder<'a> {
        let handle = FuncHandle::get("SMA").expect("SMA handle");
        let mut holder = ParamHolder::new(&handle).expect("param holder");
        holder.set_input_real(0, input).expect("bind close");
        holder.set_opt_input_integer(0, 3).expect("set period");
        holder.set_output_real(0, output).expect("bind output");
        holder
    }

    /// Function metadata and a dynamic SMA invocation agree with the static indicator.
    #[test]
    fn dynamic_sma_matches_native_values() {
        let handle = FuncHandle::get("SMA").expect("SMA handle");
        let info = handle.info().expect("SMA info");
        assert_eq!(
            (info.name.as_str(), info.group.as_str()),
            ("SMA", "Overlap Studies")
        );
        let input_info = handle.input_parameter_info(0).expect("input info");
        assert_eq!(input_info.parameter_type, InputParameterType::Real);
        let opt = handle.opt_input_parameter_info(0).expect("opt input info");
        assert_eq!(opt.parameter_type, OptInputParameterType::IntegerRange);
        let OptInputDataSet::IntegerRange(range) = opt.data_set else {
            panic!("SMA period should use an integer range");
        };
        assert!(range.min <= 2 && range.max >= 1_000);
        let output_info = handle.output_parameter_info(0).expect("output info");
        assert_eq!(output_info.parameter_type, OutputParameterType::Real);

        let input = [1.0_f64, 2.0, 3.0, 4.0, 5.0];
        let mut output = [0.0_f64; 5];
        let holder = sma_holder(&input, &mut output);
        assert_eq!(holder.lookback().expect("lookback"), 2);
        let range = holder.call(0, 4).expect("call SMA");
        assert_eq!((range.begin, range.count), (2, 3));
        drop(holder);
        assert!((output[0] - 2.0).abs() < 1e-12);
        assert!((output[1] - 3.0).abs() < 1e-12);
        assert!((output[2] - 4.0).abs() < 1e-12);
    }

    /// Output buffers shorter than the requested span are rejected before reaching C.
    #[test]
    fn short_output_buffer_is_rejected() {
        let input = [1.0_f64, 2.0, 3.0, 4.0, 5.0];
        let mut output = [0.0_f64; 3];
        let holder = sma_holder(&input, &mut output);
        assert_eq!(
            holder.call(0, 4),
            Err(Error::OutputBufferTooSmall {
                required: 5,
                actual: 3
            })
        );
    }
}
