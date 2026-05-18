# Architecture Map — qsv-dateparser

A pointer index for `find_symbol` / `replace_symbol_body` work. Line numbers
are **0-based**, matching Serena's tool output.

## `src/lib.rs` (~776 lines)
Public API and integration tests.
- `MIDNIGHT` (const) — default `NaiveTime` used across parsers.
- Functions:
  - `parse(input)` — Local-tz wrapper.
  - `parse_with_preference(input, prefer_dmy)` — Utc + DMY preference.
  - `parse_with_timezone(input, tz)` — custom tz.
  - `parse_with_preference_and_timezone(input, prefer_dmy, tz)`.
  - `parse_with(input, tz, default_time)` — base builder.
- Struct `DateTimeUtc` + `impl FromStr for DateTimeUtc`.
- `mod tests` — large block of round-trip parse assertions.

## `src/datetime.rs` (~1354 lines) — the core
Top-level:
- `fn regex` — `OnceLock`-backed regex factory (`regex!` macro wraps this).
- `struct Parse<'z, Tz2>` (lines 17–22)
  - fields: `tz`, `default_time`, `prefer_dmy`.
- `impl<'z, Tz2> Parse<'z, Tz2>` (lines 24–681). Methods, in dispatch order:

  | Method | Lines | Notes |
  |---|---|---|
  | `new`                 | 28–36   | constructor |
  | `new_with_preference` | 43–55   | constructor + DMY flag |
  | `prefer_dmy`          | 38–41   | setter |
  | `parse`               | 57–70   | top-level dispatch (see parse order) |
  | `ymd_family`          | 72–87   | rfc3339 first, then ymd variants |
  | `month_mdy_family`    | 89–101  | |
  | `month_dmy_family`    | 103–112 | |
  | `slash_mdy_family`    | 114–132 | |
  | `slash_ymd_family`    | 134–141 | |
  | `unix_timestamp`      | 143–160 | uses `fast_float2` |
  | `rfc3339`             | 162–171 | `DateTime::parse_from_rfc3339` |
  | `rfc2822`             | 173–181 | |
  | `ymd_hms`             | 183–210 | |
  | `ymd_hms_z`           | 212–247 | pre-filter: `len<17` or byte[10] not ws |
  | `ymd`                 | 249–269 | |
  | `ymd_z`               | 271–303 | |
  | `month_ymd`           | 305–326 | |
  | `month_mdy_hms`       | 328–354 | |
  | `month_mdy_hms_z`     | 356–403 | pre-filter: `len<20` + `has_year` scan |
  | `month_mdy`           | 405–435 | |
  | `month_dmy_hms`       | 437–464 | pre-filter: no `:` short-circuit |
  | `month_dmy`           | 466–502 | pre-filter: 4-digit trailing year |
  | `slash_mdy_hms`       | 504–540 | |
  | `slash_dmy_hms`       | 542–578 | |
  | `slash_mdy`           | 580–604 | |
  | `slash_dmy`           | 606–630 | |
  | `slash_ymd_hms`       | 632–657 | |
  | `slash_ymd`           | 659–680 | |
- `mod tests` — extensive per-parser unit tests.

## `src/timezone.rs` (~156 lines)
Free functions:
- `parse(s)` — entry point; resolves numeric offsets and named zones.
- `parse_offset_2822(s)` — RFC2822-style offsets.
- `parse_offset_internal(s)` — shared internal logic.
- Helpers: `equals`, `colon_or_space`.
- `mod tests`.

## `benches/parse.rs` (~109 lines)
Criterion benches (`harness = false` in `Cargo.toml`). Bench group name is
referenced by the `/bench-compare` skill.

## `examples/`
- `parse.rs`, `parse_with.rs`, `parse_with_timezone.rs`,
  `convert_to_pacific.rs`, `str_parse_method.rs`.

## `.claude/`
- `agents/performance-reviewer.md` — proactive perf reviewer for
  `src/datetime.rs`, `src/lib.rs`, `src/timezone.rs`.
- `skills/bench-compare/` — `/bench-compare` skill.
- `skills/release/` — `/release <version>` skill.
- `settings.local.json` — local Claude settings (tracked, see `.gitignore`).
