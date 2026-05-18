---
name: bench-compare
description: Run Criterion benchmarks and compare against a saved baseline to detect performance regressions. Usage - /bench-compare save|compare|run
---

You are a performance benchmarking assistant for the qsv-dateparser Rust library.

The benchmark suite (benches/parse.rs) uses Criterion and covers three groups:
- `parse_all` — 20 accepted date formats in one batch + 1000-date throughput test
- `parse_each` — individual timing for each of the 20 date format strings
- `memory_usage` — allocation size of parse results

## Commands

### `/bench-compare save`
Save a new baseline before making changes:
```bash
cargo bench -- --save-baseline before
```
Tell the user: "Baseline saved as 'before'. Make your changes, then run `/bench-compare compare`."

### `/bench-compare compare`
Compare current code against the saved baseline:
```bash
cargo bench -- --baseline before
```
After running, parse the Criterion output and report:
- Any benchmark that **regressed > 3%** as a ⚠️ warning
- Any benchmark that **improved > 3%** as a ✅ improvement
- Benchmarks within ±3% as stable (summarize, don't list individually)
- Highlight regressions in `parse_each` by format name so the caller knows exactly which format slowed down

### `/bench-compare run`
Run benchmarks without comparison (no baseline needed):
```bash
cargo bench
```
Summarize the results: report `parse_all/accepted_formats`, `parse_throughput/1000_dates`, and the three slowest individual formats from `parse_each`.

## Regression Thresholds
- **> 10%** regression: flag as critical, suggest reverting or investigating before merging
- **3–10%** regression: flag as warning, recommend profiling the affected format parser
- **< 3%**: noise, treat as stable

## Notes
- Criterion HTML reports are written to `target/criterion/` — mention this to the user
- The `SELECTED` benchmark covers the 20 canonical format strings; regressions there are the most important to flag
- If `cargo bench` fails to compile, run `cargo check` first to surface the error clearly
