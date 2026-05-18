# Task Completion Checklist

Run these before declaring a coding task done. They mirror what CI runs on
push/PR (`.github/workflows/ci.yml`).

1. **Format check**
   ```bash
   cargo fmt --all -- --check
   ```
   Fix with `cargo fmt --all` if it complains.

2. **Clippy (treats warnings as errors)**
   ```bash
   cargo clippy --workspace --tests --all-features -- -D warnings
   ```
   Common trip-ups: `useless_vec` (use `[...]` not `vec![...]` for immutable
   fixtures), unused imports, doc-link warnings.

3. **Tests**
   ```bash
   cargo test --workspace --all-features
   ```
   Tests live both inline (`mod tests` in `src/lib.rs`, `src/datetime.rs`,
   `src/timezone.rs`).

4. **Benchmarks** (only if performance-sensitive code changed)
   ```bash
   /bench-compare compare
   ```
   (or `cargo bench` directly). The skill compares against a saved Criterion
   baseline named `before`.

5. **Performance review** (if you touched
   `src/datetime.rs`, `src/lib.rs`, or `src/timezone.rs`)
   Consider invoking the `performance-reviewer` agent under `.claude/agents/`
   — it specifically watches for regressions on the parsing hot paths.

6. **Do not commit `Cargo.lock`** — it is gitignored on purpose for this
   library crate.

7. **Commit / release**
   - Commits only on explicit user request.
   - For releases, prefer the `/release <version>` skill, which handles bump,
     test, clippy, commit, and tag in one flow.
