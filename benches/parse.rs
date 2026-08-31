use chrono::Utc;
use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use qsv_dateparser::{parse, parse_with_timezone};
use std::hint::black_box;
use std::sync::OnceLock;

/// The measured entry point.
///
/// Everything here previously went through `parse()`, which resolves against
/// `Local`. That is not what this crate is optimized for: qsv parses with an
/// explicit `Utc`, and `Local`'s offset lookup costs roughly 200 ns per call —
/// about two thirds of a date-only parse. It therefore dominated every
/// measurement in this file and shrank the apparent effect of any change to
/// the parsing itself by roughly 3x.
///
/// `bench_timezone_overhead` keeps the two side by side so the gap stays
/// visible instead of being silently folded into every other number.
#[inline]
fn p(input: &str) -> bool {
    parse_with_timezone(black_box(input), &Utc).is_ok()
}

static SELECTED: OnceLock<Vec<&'static str>> = OnceLock::new();
static LARGE_DATASET: OnceLock<Vec<&'static str>> = OnceLock::new();

fn bench_parse_all(c: &mut Criterion) {
    SELECTED
        .set(vec![
            "2017-11-25T22:34:50Z",          // rfc3339
            "Wed, 02 Jun 2021 06:31:39 GMT", // rfc2822
            "2019-11-29 08:08:05-08",        // postgres_timestamp
            "2021-04-30 21:14:10",           // ymd_hms
            "2017-11-25 13:31:15 PST",       // ymd_hms_z
            "2021-02-21",                    // ymd
            "2021-02-21 PST",                // ymd_z
            "May 27 02:45:27",               // month_md_hms
            "May 8, 2009 5:57:51 PM",        // month_mdy_hms
            "May 02, 2021 15:51 UTC",        // month_mdy_hms_z
            "2021-Feb-21",                   // month_ymd
            "May 25, 2021",                  // month_mdy
            "14 May 2019 19:11:40.164",      // month_dmy_hms
            "1 July 2013",                   // month_dmy
            "03/19/2012 10:11:59",           // slash_mdy_hms
            "8/8/1965 01:00:01 PM",          // slash_mdy_hms, AM/PM (NYC 311 shape)
            "08/21/71",                      // slash_mdy
            "2012/03/19 10:11:59",           // slash_ymd_hms
            "2014/3/31",                     // slash_ymd
            "2014.03.30",                    // dot_mdy_or_ymd
            "171113 14:14:20",               // mysql_log_timestamp
        ])
        .unwrap();

    // Generate a large dataset for throughput testing
    LARGE_DATASET
        .set(
            (0..1000)
                .map(|i| {
                    let year = 2000 + (i % 24);
                    let month = 1 + (i % 12);
                    let day = 1 + (i % 28);
                    let hour = i % 24;
                    let minute = i % 60;
                    let second = i % 60;
                    format!(
                        "{:04}-{:02}-{:02} {:02}:{:02}:{:02}",
                        year, month, day, hour, minute, second
                    )
                })
                .map(|s| Box::leak(s.into_boxed_str()) as &'static str)
                .collect(),
        )
        .unwrap();

    c.bench_with_input(
        BenchmarkId::new("parse_all", "accepted_formats"),
        &SELECTED.get().unwrap(),
        |b, all| {
            b.iter(|| {
                for date_str in all.iter() {
                    black_box(p(date_str));
                }
            })
        },
    );

    // Benchmark throughput with large dataset
    c.bench_with_input(
        BenchmarkId::new("parse_throughput", "1000_dates"),
        &LARGE_DATASET.get().unwrap(),
        |b, all| {
            b.iter(|| {
                for date_str in all.iter() {
                    black_box(p(date_str));
                }
            })
        },
    );
}

fn bench_parse_each(c: &mut Criterion) {
    let mut group = c.benchmark_group("parse_each");
    for date_str in SELECTED.get().unwrap().iter() {
        group.bench_with_input(*date_str, *date_str, |b, input| b.iter(|| p(input)));
    }
    group.finish();
}

// Benchmark the FAILURE hot path: non-date string columns. This is the dominant
// cost of `qsv stats --infer-dates` on real data, where most columns are not
// dates and every value runs the full dispatch chain before failing.
static FAILURES: OnceLock<Vec<&'static str>> = OnceLock::new();

fn bench_parse_failures(c: &mut Criterion) {
    FAILURES
        .set(
            (0..1000)
                .map(|i| {
                    // Mirrors the qsv-dateparser-opt repro column `category_value_%d`.
                    // The '_' is rejected by the structural pre-filter before any regex.
                    format!("category_value_{}", i % 500)
                })
                .map(|s| Box::leak(s.into_boxed_str()) as &'static str)
                .collect(),
        )
        .unwrap();

    c.bench_with_input(
        BenchmarkId::new("parse_failures", "1000_nondate_strings"),
        &FAILURES.get().unwrap(),
        |b, all| {
            b.iter(|| {
                for date_str in all.iter() {
                    black_box(p(date_str));
                }
            })
        },
    );
}

// Benchmark the OTHER failure hot path: non-date *word* columns (status,
// category, region values) that use only date-valid bytes, so they pass the
// structural pre-filter and run the full gate chain + `unix_timestamp` +
// `rfc2822` before failing. This is the path the family reorder and the
// `unix_timestamp` first-byte pre-check operate on.
static WORD_FAILURES: OnceLock<Vec<&'static str>> = OnceLock::new();

fn bench_parse_word_failures(c: &mut Criterion) {
    const WORDS: [&str; 10] = [
        "pending", "active", "north", "south", "category", "region", "status", "approved",
        "rejected", "unknown",
    ];
    WORD_FAILURES
        .set((0..1000).map(|i| WORDS[i % WORDS.len()]).collect())
        .unwrap();

    c.bench_with_input(
        BenchmarkId::new("parse_word_failures", "1000_nondate_words"),
        &WORD_FAILURES.get().unwrap(),
        |b, all| {
            b.iter(|| {
                for date_str in all.iter() {
                    black_box(p(date_str));
                }
            })
        },
    );
}

// Cost of the timezone the caller picks, holding the parsing identical.
//
// `Local` has to resolve a zone offset; `Utc` does not. Splitting this out
// keeps the difference measurable on its own, and stops it from being folded
// into every other benchmark in this file — which is what happened while they
// all went through `parse()`.
//
// The date-only input is the interesting one: those parsers consult
// `Utc::now()` to derive a default time-of-day, so they pay for the zone twice.
fn bench_timezone_overhead(c: &mut Criterion) {
    let mut group = c.benchmark_group("timezone_overhead");
    for input in ["2021-02-21", "2021-04-30 21:14:10"] {
        group.bench_with_input(BenchmarkId::new("utc", input), input, |b, input| {
            b.iter(|| parse_with_timezone(black_box(input), &Utc).is_ok());
        });
        group.bench_with_input(BenchmarkId::new("local", input), input, |b, input| {
            b.iter(|| parse(black_box(input)).is_ok());
        });
    }
    group.finish();
}

// Benchmark memory usage
fn bench_memory_usage(c: &mut Criterion) {
    c.bench_function("memory_usage", |b| {
        b.iter(|| {
            let mut total = 0;
            for date_str in SELECTED.get().unwrap().iter() {
                let result = parse_with_timezone(black_box(date_str), &Utc);
                total += std::mem::size_of_val(&result);
            }
            total
        })
    });
}

criterion_group!(
    benches,
    bench_parse_all,
    bench_parse_each,
    bench_parse_failures,
    bench_parse_word_failures,
    bench_timezone_overhead,
    bench_memory_usage
);
criterion_main!(benches);
