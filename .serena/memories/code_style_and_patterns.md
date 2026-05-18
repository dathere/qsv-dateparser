# Code Style & Patterns

## General
- Rust edition **2024**, MSRV **1.93**.
- `cargo fmt` is enforced in CI — run before committing.
- `cargo clippy --workspace --tests --all-features -- -D warnings` must pass.
  - Common gotcha: `vec![...]` that never mutates should be an array literal
    `[...]` — clippy flags this as `useless_vec`.
- No custom rustfmt.toml; defaults are used.

## Hot-path patterns in `src/datetime.rs`

### 1. Two-layer pre-filter pattern (the single most important pattern)
This is documented in the root `CLAUDE.md` and is THE established pattern for
adding/optimizing parsers. Each parse attempt goes through three gates:

1. **Family regex gate** (dispatch in `parse()` / `*_family()` methods) — picks
   which family of parsers to try (slash_mdy / ymd / month_dmy / …).
2. **Byte pre-filter** inside each individual parser — cheap O(1) byte checks
   (length, presence of `:`, digit at a known offset, etc.) to bail out *before*
   compiling/running the parser's own regex.
3. **Specific regex / chrono parse** — only reached when the pre-filter passes.

Examples of pre-filters currently in the code (track these — they're easy to
break):
- `ymd_hms_z`: `input.len() < 17 || !input.as_bytes()[10].is_ascii_whitespace()`.
- `month_mdy_hms_z`: `input.len() < 20` + an O(n) `has_year` scan.
- `month_dmy_hms`: `!input.as_bytes().contains(&b':')` — skip if no colon.
- `month_dmy`: 4-digit trailing-year detection to skip the `%d %B %y` attempt.

Add new pre-filters following the same shape: short-circuit on `!matches` before
any regex.

### 2. Regex compilation
- Custom `regex!` macro in `src/datetime.rs` wraps each regex in `OnceLock` so
  every pattern is compiled exactly once across the program's lifetime.
- Keep capture groups to the minimum needed.

### 3. Numeric parsing
- Use `fast_float2::parse()` for floats (notably unix timestamps),
  **not** `str::parse::<f64>()`.

### 4. `#[inline]`
- All family dispatch methods (`ymd_family`, `month_dmy_family`, …) and the
  individual parsers carry `#[inline]`. Preserve this when adding new ones.

### 5. Parse order
Defined in `Parse::parse()`:
```
rfc2822
→ unix_timestamp
→ slash_mdy_family
→ slash_ymd_family
→ ymd_family             (rfc3339 first inside this family)
→ month_ymd
→ month_mdy_family
→ month_dmy_family
```
RFC3339 lives inside `ymd_family` and uses `chrono::DateTime::parse_from_rfc3339`.

### 6. Family dispatch idiom
Each `*_family` method chains attempts with `or_else()`:
```
self.first_parser().or_else(|_| self.second_parser()).or_else(...)
```
This composes cheaply because the early ones short-circuit on the byte
pre-filter without paying regex cost.

## Naming
- Parsers are named for the format they handle: `ymd_hms_z`, `month_dmy`,
  `slash_mdy_hms`, etc. Keep new parsers in the same `verb_subject_modifier` style.
- DMY/MDY ambiguity is controlled by `prefer_dmy: bool` on `Parse`.

## Comments
- Comments explain **why**, not what. Pre-filters often have a 1-line comment
  noting which inputs they short-circuit and why it's safe.
- Don't add task/PR references in source comments.
