# Suggested Commands

The system is **Darwin** (macOS); standard BSD-flavored utilities apply
(`ls`, `find`, `grep`, etc.). All commands below are run from the repo root
`/Users/joelnatividad/GitHub/qsv-dateparser`.

## Build
```bash
cargo build               # debug build
cargo build --release     # release build
```

## Test
```bash
cargo test                            # all tests
cargo test test_name                  # run a single test by name substring
cargo test --workspace --all-features # what CI runs
```

## Lint / Format (required before commit — CI enforces both)
```bash
cargo fmt --all -- --check
cargo clippy --workspace --tests --all-features -- -D warnings
```
To auto-fix formatting: `cargo fmt --all`.

## Benchmarks
Criterion harness in `benches/parse.rs` (bench name = `parse`).
```bash
cargo bench               # run all benchmarks
cargo bench parse         # run the 'parse' bench specifically
```
There is also a custom skill workflow:
```bash
/bench-compare save       # save current state as baseline 'before'
/bench-compare compare    # compare against the saved baseline
/bench-compare run        # plain run
```
The baseline was reset to current state after the 0.14.0 perf work.

## Release
A custom slash command handles the release checklist (bump, test, clippy,
commit, tag):
```bash
/release <version>
```

## Run examples
```bash
cargo run --example parse
cargo run --example parse_with
cargo run --example parse_with_timezone
cargo run --example convert_to_pacific
cargo run --example str_parse_method
```

## Git
- Repo is at `https://github.com/dathere/qsv-dateparser` (`dathere` org).
- `Cargo.lock` is **gitignored** (library crate convention) — do not stage it.
- Default branch: `main`.

## macOS-specific
- File listing: `ls -la`.
- Recursive search: prefer `rg` (ripgrep) if installed, else
  `grep -RIn pattern .`.
- BSD `sed` requires `sed -i '' …` (note the empty `''` after `-i`). When
  doing multi-line or regex-backref edits, use Serena's edit tools rather than
  `sed`.
