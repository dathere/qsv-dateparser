# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build and Test Commands

```bash
# Build
cargo build
cargo build --release

# Run all tests
cargo test

# Run a specific test
cargo test test_name

# Run benchmarks
cargo bench
```

## Architecture

qsv-dateparser is a performance-optimized Rust library for parsing date strings into `chrono::DateTime<Utc>`. It is a fork of [dateparser](https://github.com/waltzofpearls/belt/tree/main/dateparser), optimized for use in [qsv](https://github.com/jqnatividad/qsv).

### Core Structure

- **`src/lib.rs`**: Public API entry points
  - `parse()` - Parse with Local timezone assumption
  - `parse_with_preference()` - Parse with DMY/MDY preference
  - `parse_with_timezone()` - Parse with custom timezone
  - `parse_with_preference_and_timezone()` - Parse with DMY/MDY preference and custom timezone
  - `parse_with()` - Parse with custom timezone and default `NaiveTime`
  - `DateTimeUtc` - Wrapper implementing `FromStr` for `str::parse()` usage

- **`src/datetime.rs`**: Core parsing logic in `Parse` struct
  - Uses regex to detect format families, then tries specific parsers
  - Parse order: rfc2822 -> unix_timestamp -> slash formats -> ymd formats -> month formats
  - Each format family has a detection regex followed by specific format attempts using `or_else()` chains
  - Regexes are compiled once using `OnceLock` via a custom macro

- **`src/timezone.rs`**: Timezone offset parsing
  - Handles numeric offsets (+0800, +08:00) and named zones (PST, UTC, GMT)

### Key Design Decisions

1. **Format detection uses regex families**: Before trying specific parsers, a quick regex check determines which format family to try, avoiding unnecessary parsing attempts. Within each family, individual parsers apply cheap byte pre-filters (e.g. checking for `:`, input length, or trailing digit patterns) *before* running their regex, eliminating regex overhead on non-matching inputs. This two-layer approach — family regex gate, then byte pre-filter, then specific regex — is the established pattern for adding new parsers or optimizing existing ones.

2. **DMY preference**: The `prefer_dmy` flag controls whether `dd/mm/yyyy` or `mm/dd/yyyy` is tried first for ambiguous slash-separated dates.

3. **RFC3339 is parsed inside `ymd_family`**: `rfc3339()` is tried first within `ymd_family` using `chrono::DateTime::parse_from_rfc3339()`. The parse order is: rfc2822 → unix_timestamp → slash_mdy_family → slash_ymd_family → ymd_family (rfc3339 first) → month_ymd → month_mdy_family → month_dmy_family.

4. **Performance focus**: Uses `fast-float2` for timestamp parsing, minimal regex capture groups, and `#[inline]` annotations on hot paths.
