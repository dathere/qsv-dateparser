use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use qsv_dateparser::parse;
use std::sync::OnceLock;

static SELECTED: OnceLock<Vec<&'static str>> = OnceLock::new();

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
            "08/21/71",                      // slash_mdy
            "2012/03/19 10:11:59",           // slash_ymd_hms
            "2014/3/31",                     // slash_ymd
            "2014.03.30",                    // dot_mdy_or_ymd
            "171113 14:14:20",               // mysql_log_timestamp
        ])
        .unwrap();
    c.bench_with_input(
        BenchmarkId::new("parse_all", "accepted_formats"),
        &SELECTED.get().unwrap(),
        |b, all| {
            b.iter(|| {
                for date_str in all.iter() {
                    let _ = parse(*date_str);
                }
            })
        },
    );
}

fn bench_parse_each(c: &mut Criterion) {
    let mut group = c.benchmark_group("parse_each");
    for date_str in SELECTED.get().unwrap().iter() {
        group.bench_with_input(*date_str, *date_str, |b, input| b.iter(|| parse(input)));
    }
    group.finish();
}

criterion_group!(benches, bench_parse_all, bench_parse_each);
criterion_main!(benches);
