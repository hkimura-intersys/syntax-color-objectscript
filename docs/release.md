# Release and Publish Guide

This guide publishes all three crates to crates.io in dependency order:

1. `highlight-spans`
2. `theme-engine`
3. `render-ansi`

## Prerequisites

- crates.io account
- crates.io API token
- clean local build (`cargo test` passes)

## 1) Authenticate Cargo

```bash
cargo login <CRATES_IO_TOKEN>
```

Alternative (CI/non-interactive):

```bash
export CARGO_REGISTRY_TOKEN=<CRATES_IO_TOKEN>
```

## 2) (Optional) Check crate name availability

```bash
cargo search highlight-spans --limit 1
cargo search theme-engine --limit 1
cargo search render-ansi --limit 1
```

## 3) Bump version for a new release

Edit workspace version in root `Cargo.toml`:

```toml
[workspace.package]
version = "0.4.0"
```

For the next release, change to `0.1.1`, `0.2.0`, etc.

## 4) Dry-run publish checks

Run from repository root:

```bash
cargo publish -p highlight-spans --dry-run
cargo publish -p theme-engine --dry-run
cargo publish -p render-ansi --dry-run
```

## 5) Publish in order

```bash
cargo publish -p highlight-spans
cargo publish -p theme-engine
cargo publish -p render-ansi
```

If crates.io index propagation is still catching up, wait 1-2 minutes and retry `render-ansi`.

## 6) Post-publish verification

```bash
cargo search highlight-spans --limit 1
cargo search theme-engine --limit 1
cargo search render-ansi --limit 1
```

## Notes

- `render-ansi` uses local `path` dependencies for workspace development and explicit `version` constraints for crates.io publishing.
- Publish order matters because `render-ansi` depends on the other two crates.
