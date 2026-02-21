---
name: release
description: Prepare and publish a new qsv-dateparser release. Usage - /release <version> (e.g. /release 0.14.0)
disable-model-invocation: true
---

You are a release assistant for the qsv-dateparser Rust library. Follow these steps exactly and stop if any step fails.

## Steps

### 1. Confirm the version argument
If no version was provided, ask the user: "Which version number should I release? (e.g. 0.14.0)"

### 2. Check working tree is clean
```bash
git status --short
```
If any uncommitted changes exist, stop and tell the user to commit or stash them first.

### 3. Confirm no existing tag
```bash
git tag --list "<version>"
```
If the tag already exists, stop and warn the user.

### 4. Bump version in Cargo.toml
Edit the `version` field in `[package]` in `Cargo.toml` to the new version string.

### 5. Refresh Cargo.lock
```bash
cargo build
```
This updates `Cargo.lock` to reflect the new version. Stop if it fails.

### 6. Run the full test suite
```bash
cargo test
```
Stop if any test fails — do not proceed to commit.

### 7. Run clippy
```bash
cargo clippy --workspace --tests --all-features -- -D warnings
```
Stop if there are any warnings — do not proceed to commit.

### 8. Stage and commit
```bash
git add Cargo.toml Cargo.lock
git commit -m "<version> release"
```
The commit message format matches the project convention (e.g. `0.13.0 release`).

### 9. Create an annotated tag
```bash
git tag -a <version> -m "<version>"
```

### 10. Show summary and next steps
Print a summary:
- Version bumped to: `<version>`
- Commit: show the short SHA from `git log -1 --oneline`
- Tag: `<version>`

Then tell the user:
> Review the commit and tag above, then run the following to publish:
> ```bash
> git push origin main
> git push origin <version>
> cargo publish
> ```
> These are NOT run automatically — confirm before pushing.
