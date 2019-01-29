#[macro_use]
extern crate criterion;
extern crate bitap;

use bitap::reference::BitapFast;
use criterion::Criterion;

static BENCH_PATTERN: &str = "bitap";
const BENCH_TEXT: &'static [&'static str] = &[
    "------------------------------------------------",
    "bitap-------------------------------------------",
    "--------------------bitap-----------------------",
    "-------------------------------------------bitap",
];

fn criterion_benchmark(c: &mut Criterion) {
    // TODO: Generate test cases randomly. Vary text length, pattern length,
    // match location (start, end, none). Benchmark both including pattern
    // mask creation time and amortized.
    let s = BitapFast::new(BENCH_PATTERN);
    for (i, txt) in BENCH_TEXT.iter().enumerate() {
        c.bench_function(&format!("bitap_{}", i + 1), move |b| b.iter(|| s.find(txt)));
        c.bench_function(&format!("bitap_iter{}", i + 1), move |b| {
            b.iter(|| s.find_iter(txt).next())
        });
        c.bench_function(&format!("baseline_{}", i + 1), move |b| {
            b.iter(|| txt.find(BENCH_PATTERN))
        });
    }
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
