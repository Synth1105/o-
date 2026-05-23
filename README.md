# o-

Multi-engine JavaScript runtime implemented in Rust.

## Workspace crates

- `o-`: CLI and top-level library crate.
- `o-core`: shared runtime traits and error types.
- `jstd`: JavaScript standard-library surface used by toolchains.
- `o-toolchain-javascriptcore`: JavaScriptCore integration.
- `o-toolchain-spidermonkey`: SpiderMonkey integration.
- `o-toolchain-v8`: V8 integration.

## Release process

This repository is configured for [`release-plz`](https://release-plz.dev/).

- Push regular changes to `main`.
- `release-plz` opens or updates a release PR with version bumps and changelog changes.
- Merging that PR publishes unpublished crates and creates git tags and GitHub releases.

The release workflow expects a `CARGO_REGISTRY_TOKEN` secret with publish access to crates.io.
