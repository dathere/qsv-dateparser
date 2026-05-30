# Architecture Map — qsv-dateparser

A pointer index for `find_symbol` / `replace_symbol_body` work. Line numbers
are **0-based**, matching Serena's tool output.

> NOTE: After the 0.15.x parse-dispatch optimization (structural pre-filter +
> family reorder), absolute line numbers in `src/datetime.rs` shifted by ~+20
> for symbols below `parse`. Prefer `find_symbol` by name over the line numbers
> below, which are approximate until re-derived.

## `src/lib.rs` (~830 lines)
Public API and integration tests.
- `MIDNIGHT` (const) — default `NaiveTime` used across parsers.
- Functions:
  - `parse(input)` — Local-tz wrapper.
  - `parse_with_preference(input, prefer_dmy)` — Utc + DMY preference.
  - `parse_with_timezone(input, tz)` — custom tz.
  - `parse_with_preference_and_timezone(input, prefer_dmy, tz)`.
  - `parse_with(input, tz, default_time)` — base builder.
- Struct `DateTimeUtc` + `impl FromStr for DateTimeUtc`.
- `mod tests` — round-trip parse assertions. Includes
  `prefilter_rejects_non_date_strings` (pins the structural pre-filter).

## `src/datetime.rs` — the core
Top-level (above `struct Parse`):
- `regex!` macro — `OnceLock`-backed regex factory, `.unicode(false)`.
- `build_date_byte_table()` (const fn) — builds the 256-entry `DATE_BYTE` LUT.
- `static DATE_BYTE: [bool; 256]` — valid date bytes (alnum, ASCII ws, `- + / : . ,`).
- `fn cannot_be_date(input)` — whole-input structural pre-filter; bails before
  the dispatch chain when any byte cannot appear in a date (`_`, `#`, non-ASCII…).

- `struct Parse<'z, Tz2>` — fields `tz`, `default_time`, `prefer_dmy`.
- `impl<'z, Tz2> Parse<'z, Tz2>`. Methods of note:
  - `new`, `new_with_preference`, `prefer_dmy` — constructors/setter.
  - `parse` — top-level dispatch. Runs `cannot_be_date` first, then the
    `or_else` chain (see Parse Order below).
  - Family dispatchers: `ymd_family` (rfc3339 first), `month_mdy_family`,
    `month_dmy_family`, `slash_mdy_family`, `slash_ymd_family`.
  - Ungated parsers (run last): `unix_timestamp` (`fast_float2`), `rfc2822`.
  - Leaf parsers: `rfc3339`, `ymd_hms`, `ymd_hms_z`, `ymd`, `ymd_z`,
    `month_ymd`, `month_mdy_hms`, `month_mdy_hms_z`, `month_mdy`,
    `month_dmy_hms`, `month_dmy`, `slash_mdy_hms`, `slash_dmy_hms`,
    `slash_mdy`, `slash_dmy`, `slash_ymd_hms`, `slash_ymd`.
  - Leaf pre-filters (cheap byte checks before the heavy parse): `ymd_hms_z`
    (`len<17` or byte[10] not ws); `month_mdy_hms_z` (`len<20` + `has_year`
    scan); `month_dmy_hms` (no `:` short-circuit); `month_dmy` (4-digit trailing
    year); `unix_timestamp` (first byte must be digit/`+`/`-`/`.`/`i`/`I`/`n`/`N`
    — the only leads `fast_float2` accepts); `rfc2822` (must contain `:`, since
    every RFC2822 datetime has a time-of-day).
- `mod tests` — extensive per-parser unit tests.

## Parse Order (current)
`cannot_be_date` gate → slash_mdy_family → slash_ymd_family →
ymd_family (rfc3339 first) → month_ymd → month_mdy_family → month_dmy_family →
unix_timestamp → rfc2822.

Reorder is result-preserving: floats match no family gate (so still reach
`unix_timestamp`); rfc2822 inputs always carry a tz which the `$`-anchored
`month_dmy_*` regexes reject, and `month_dmy_*` only succeeds without a tz which
makes rfc2822 fail — the two are mutually exclusive.

## `src/timezone.rs` (~156 lines)
- `parse(s)` — entry; numeric offsets + named zones.
- `parse_offset_2822(s)`, `parse_offset_internal(s)`, helpers `equals`,
  `colon_or_space`. `mod tests`.

## `benches/parse.rs`
Criterion benches (`harness = false`). Groups: `parse_all`, `parse_each`,
`parse_failures` (non-date hot path: `category_value_N` strings, killed by the
byte pre-filter), `parse_word_failures` (valid-byte word non-dates that run the
full chain), `memory_usage`. Referenced by the `/bench-compare` skill.

## `.claude/`
- `agents/performance-reviewer.md` — proactive perf reviewer for
  `src/datetime.rs`, `src/lib.rs`, `src/timezone.rs`.
- `skills/bench-compare/`, `skills/release/`.
- `settings.local.json` — local Claude settings (tracked).
