# qsv-dateparser — Project Overview

## Purpose
`qsv-dateparser` is a performance-optimized Rust library for parsing date strings
into `chrono::DateTime<Utc>`. It is a fork of Rollie Ma's
[dateparser](https://github.com/waltzofpearls/belt/tree/main/dateparser), trimmed
to a subset of formats and tuned for use inside
[qsv](https://github.com/jqnatividad/qsv).

Distinguishing features vs. upstream dateparser:
- Subset of formats (drops obscure ones) for higher throughput.
- Adds **DMY** parsing support via `parse_with_preference`.
- Heavy use of byte pre-filters before regex (see `code_style_and_patterns`).

## Crate metadata (Cargo.toml)
- Name: `qsv-dateparser`
- Current version: `0.14.0`
- Edition: `2024`
- `rust-version`: `1.93`
- License: MIT
- Author: Joel Natividad <joel@dathere.com>
- Repository field still says `github.com/jqnatividad/qsv-dateparser`, but the
  org has moved to **`dathere`** — repo URL: <https://github.com/dathere/qsv-dateparser>.

## Dependencies
- `anyhow = 1.0` — error type for parse APIs.
- `chrono = 0.4` (default-features off; `clock`, `std` only).
- `fast-float2 = 0.2` — fast float parsing for unix timestamps.
- `regex = 1` (default-features off; `std`, `perf`).

Dev-dependencies:
- `chrono-tz = 0.10`
- `criterion = 0.8` (with `html_reports`) — Criterion harness for `benches/parse.rs`.

## Public API (src/lib.rs)
Top-level functions:
- `parse(input) -> Result<DateTime<Utc>>` — Local timezone assumption.
- `parse_with_preference(input, prefer_dmy)` — DMY/MDY preference, Utc.
- `parse_with_timezone(input, tz)` — custom timezone.
- `parse_with_preference_and_timezone(input, prefer_dmy, tz)`.
- `parse_with(input, tz, default_time)` — custom tz + default `NaiveTime`.
- `DateTimeUtc` struct that implements `FromStr`, so callers can write
  `"...".parse::<DateTimeUtc>()`.

`MIDNIGHT` is the shared `NaiveTime` default.

## Source layout (~2.4k LOC total)
- `src/lib.rs` (~776 lines): public API + integration tests in `mod tests`.
- `src/datetime.rs` (~1354 lines): `Parse<'z, Tz2>` struct and all parsing logic.
- `src/timezone.rs` (~156 lines): named-zone / numeric-offset parsing
  (`parse`, `parse_offset_2822`, `parse_offset_internal`, helpers).
- `benches/parse.rs`: Criterion benchmark suite.
- `examples/`: usage examples — `parse.rs`, `parse_with.rs`,
  `parse_with_timezone.rs`, `convert_to_pacific.rs`, `str_parse_method.rs`.

## CI
`.github/workflows/ci.yml` matrix: Linux/macOS/Windows × stable/nightly.
Jobs: `check`, `test`, `fmt` (rustfmt check), `clippy`
(`--workspace --tests --all-features -- -D warnings`).
