//! Caller-owned batch outputs and warmed native streams through public wrappers.
//! Allocation checks observe the Rust global allocator, not allocations inside native C.
use std::{
    alloc::{GlobalAlloc, Layout, System},
    cell::Cell,
    hint::black_box,
    time::{Duration, Instant},
};

use criterion::{Bencher, Criterion, criterion_group, criterion_main};
use ta_lib::{FeatureSet, MAType, TaStream, *};

struct CountingAllocator;
thread_local! {
    static ACTIVE: Cell<bool> = const { Cell::new(false) };
    static ALLOCATIONS: Cell<usize> = const { Cell::new(0) };
}
fn record_allocation() {
    if ACTIVE.try_with(Cell::get).unwrap_or(false) {
        ALLOCATIONS.with(|value| value.set(value.get() + 1));
    }
}
// SAFETY: System receives every original pointer, size and alignment unchanged.
unsafe impl GlobalAlloc for CountingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        record_allocation();
        // SAFETY: The caller supplies a valid allocation layout.
        unsafe { System.alloc(layout) }
    }
    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        record_allocation();
        // SAFETY: The caller supplies a valid allocation layout.
        unsafe { System.alloc_zeroed(layout) }
    }
    unsafe fn realloc(&self, pointer: *mut u8, layout: Layout, size: usize) -> *mut u8 {
        record_allocation();
        // SAFETY: The original live allocation and requested size satisfy GlobalAlloc.
        unsafe { System.realloc(pointer, layout, size) }
    }
    unsafe fn dealloc(&self, pointer: *mut u8, layout: Layout) {
        // SAFETY: The allocation and its original layout are forwarded unchanged.
        unsafe { System.dealloc(pointer, layout) }
    }
}
#[global_allocator]
static ALLOCATOR: CountingAllocator = CountingAllocator;

fn zero_alloc<T>(name: &str, operation: impl FnOnce() -> T) -> T {
    ALLOCATIONS.with(|value| value.set(0));
    ACTIVE.with(|active| active.set(true));
    let result = operation();
    ACTIVE.with(|active| active.set(false));
    assert_eq!(
        ALLOCATIONS.with(Cell::get),
        0,
        "{name} allocated Rust storage"
    );
    result
}
#[derive(Debug, Clone, PartialEq, TaStream)]
pub struct Incremental {
    #[ta(sma(16))]
    pub sma: f64,
    #[ta(rsi(14))]
    pub rsi: f64,
}
#[derive(Debug, Clone, PartialEq, FeatureSet)]
pub struct Factors {
    #[ta(sma(16))]
    pub sma: f64,
    #[ta(ema(16))]
    pub ema: f64,
    #[ta(rsi(14))]
    pub rsi: f64,
    #[ta(macd(12, 26, 9), out = 2)]
    pub histogram: f64,
    #[ta(bbands(20, 2.0, 2.0, MAType::Sma))]
    pub bands: (f64, f64, f64),
    #[ta(atr(14))]
    pub atr: f64,
}
struct Bars<T> {
    open: Vec<T>,
    high: Vec<T>,
    low: Vec<T>,
    close: Vec<T>,
}
fn bars() -> Bars<f64> {
    let close: Vec<_> = (0..256)
        .map(|i| 100.0 + i as f64 * 0.1 + (i % 7) as f64 * 0.05)
        .collect();
    Bars {
        open: close
            .iter()
            .enumerate()
            .map(|(i, value)| {
                if i % 7 == 0 {
                    *value
                } else if i % 2 == 0 {
                    value + 0.1
                } else {
                    value - 0.1
                }
            })
            .collect(),
        high: close.iter().map(|value| value + 0.5).collect(),
        low: close.iter().map(|value| value - 0.5).collect(),
        close,
    }
}
fn batches(c: &mut Criterion, bars: &Bars<f64>) {
    let single = Bars {
        open: bars.open.iter().map(|value| *value as f32).collect(),
        high: bars.high.iter().map(|value| *value as f32).collect(),
        low: bars.low.iter().map(|value| *value as f32).collect(),
        close: bars.close.iter().map(|value| *value as f32).collect(),
    };
    let mut a = vec![0.0; 256];
    let mut b = vec![0.0; 256];
    let mut d = vec![0.0; 256];
    macro_rules! batch {
        ($name:literal, $expression:expr) => {{
            let range = zero_alloc($name, || $expression.unwrap());
            assert!(range.count > 0);
            c.bench_function($name, |bencher| {
                bencher.iter(|| black_box($expression.unwrap()))
            });
        }};
    }
    batch!(
        "batch/f64/sma_256",
        sma_into(black_box(&bars.close), Some(16), &mut a)
    );
    batch!(
        "batch/f32/sma_256",
        sma_f32_into(black_box(&single.close), Some(16), &mut a)
    );
    batch!(
        "batch/f64/ema_256",
        ema_into(black_box(&bars.close), Some(16), &mut a)
    );
    batch!(
        "batch/f32/ema_256",
        ema_f32_into(black_box(&single.close), Some(16), &mut a)
    );
    batch!(
        "batch/f64/rsi_256",
        rsi_into(black_box(&bars.close), Some(14), &mut a)
    );
    batch!(
        "batch/f32/rsi_256",
        rsi_f32_into(black_box(&single.close), Some(14), &mut a)
    );
    batch!(
        "batch/f64/macd_256",
        macd_into(
            black_box(&bars.close),
            Some(12),
            Some(26),
            Some(9),
            &mut a,
            &mut b,
            &mut d
        )
    );
    batch!(
        "batch/f32/macd_256",
        macd_f32_into(
            black_box(&single.close),
            Some(12),
            Some(26),
            Some(9),
            &mut a,
            &mut b,
            &mut d
        )
    );
    batch!(
        "batch/f64/bbands_256",
        bbands_into(
            black_box(&bars.close),
            Some(20),
            Some(2.0),
            Some(2.0),
            Some(MAType::Sma),
            &mut a,
            &mut b,
            &mut d
        )
    );
    batch!(
        "batch/f32/bbands_256",
        bbands_f32_into(
            black_box(&single.close),
            Some(20),
            Some(2.0),
            Some(2.0),
            Some(MAType::Sma),
            &mut a,
            &mut b,
            &mut d
        )
    );
    batch!(
        "batch/f64/atr_256",
        atr_into(
            black_box(&bars.high),
            &bars.low,
            &bars.close,
            Some(14),
            &mut a
        )
    );
    batch!(
        "batch/f32/atr_256",
        atr_f32_into(
            black_box(&single.high),
            &single.low,
            &single.close,
            Some(14),
            &mut a
        )
    );
    let mut pattern = vec![0; 256];
    batch!(
        "patterns/engulfing_256",
        cdlengulfing_into(
            black_box(&bars.open),
            &bars.high,
            &bars.low,
            &bars.close,
            &mut pattern
        )
    );
    batch!(
        "patterns/doji_256",
        cdldoji_into(
            black_box(&bars.open),
            &bars.high,
            &bars.low,
            &bars.close,
            &mut pattern
        )
    );
}
// Native streams cap their cumulative bar index. Clone outside timing and bound each commit run.
fn updating<S>(b: &mut Bencher, mut setup: impl FnMut() -> S, mut update: impl FnMut(&mut S)) {
    b.iter_custom(|iterations| {
        let mut remaining = iterations;
        let mut measured = Duration::ZERO;
        while remaining > 0 {
            let count = remaining.min(4096);
            let mut stream = setup();
            let start = Instant::now();
            for _ in 0..count {
                update(&mut stream);
            }
            measured += start.elapsed();
            remaining -= count;
        }
        measured
    });
}
fn streams(c: &mut Criterion, bars: &Bars<f64>) {
    let history: Vec<f64> = (1..=16).map(f64::from).collect();
    let (mut sma, value) = SmaStream::open(&history, Some(4)).unwrap();
    assert_eq!(value, 14.5);
    assert_eq!(sma.peek(20.0).unwrap(), 16.25);
    assert_eq!(sma.value().unwrap(), 14.5);
    assert_eq!(sma.update(20.0).unwrap(), 16.25);
    let mut incremental = IncrementalStream::open(&bars.close).unwrap().0;
    let snapshot = incremental.value().unwrap();
    let preview = zero_alloc("TaStream peek", || incremental.peek(126.0).unwrap());
    assert_eq!(incremental.value().unwrap(), snapshot);
    let mut branch = incremental.try_clone().unwrap();
    assert_eq!(branch.update(126.0).unwrap(), preview);
    assert_eq!(
        zero_alloc("TaStream update", || incremental.update(126.0).unwrap()),
        preview
    );
    c.bench_function("stream/derive_update", |b| {
        updating(
            b,
            || incremental.try_clone().unwrap(),
            |stream| {
                black_box(stream.update(black_box(126.0)).unwrap());
            },
        )
    });
    c.bench_function("stream/derive_peek", |b| {
        b.iter(|| black_box(incremental.peek(black_box(127.0)).unwrap()))
    });
    let mut factors = FactorsStream::open(&bars.high, &bars.low, &bars.close)
        .unwrap()
        .0;
    let before = factors.value().unwrap();
    let preview = zero_alloc("FeatureSet peek", || {
        factors.peek(126.5, 125.5, 126.0).unwrap()
    });
    assert_eq!(factors.value().unwrap(), before);
    let mut branch = factors.try_clone().unwrap();
    assert_eq!(branch.update(126.5, 125.5, 126.0).unwrap(), preview);
    assert_eq!(
        zero_alloc("FeatureSet update", || factors
            .update(126.5, 125.5, 126.0)
            .unwrap()),
        preview
    );
    c.bench_function("features/update", |b| {
        updating(
            b,
            || factors.try_clone().unwrap(),
            |stream| {
                black_box(
                    stream
                        .update(black_box(126.5), black_box(125.5), black_box(126.0))
                        .unwrap(),
                );
            },
        )
    });
    c.bench_function("features/peek", |b| {
        b.iter(|| {
            black_box(
                factors
                    .peek(black_box(126.5), black_box(125.5), black_box(126.0))
                    .unwrap(),
            )
        })
    });
    for period in [16, 64, 256] {
        let mut stream = SmaStream::open(&bars.close, Some(period)).unwrap().0;
        zero_alloc("native SMA update/peek", || {
            black_box(stream.update(126.0).unwrap());
            black_box(stream.peek(127.0).unwrap());
        });
        c.bench_function(&format!("stream/sma_update_{period}"), |b| {
            updating(
                b,
                || stream.try_clone().unwrap(),
                |state| {
                    black_box(state.update(black_box(126.0)).unwrap());
                },
            )
        });
        c.bench_function(&format!("stream/sma_peek_{period}"), |b| {
            b.iter(|| black_box(stream.peek(black_box(127.0)).unwrap()))
        });
    }
}
fn indicators(c: &mut Criterion) {
    initialize().unwrap();
    let history = bars();
    batches(c, &history);
    streams(c, &history);
}
criterion_group! { name = benches; config = Criterion::default().sample_size(30); targets = indicators }
criterion_main!(benches);
