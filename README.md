# o-

`o-` is a Rust-based JavaScript runtime with multiple engine backends and an
experimental npm-style package manager.

## What It Does

- runs JavaScript through SpiderMonkey, V8, or JavaScriptCore
- loads a shared `jstd` bootstrap layer for common host behavior
- installs npm packages into local `node_modules`
- supports global package installs with generated `.bin` shims
- writes npm-style `package-lock.json` files
- formats runtime and package-manager failures through a shared `Report` flow

## Workspace Layout

- `o-`: CLI and top-level integration crate
- `o-core`: shared runtime traits and error types
- `jstd`: shared JavaScript bootstrap layer
- `o-toolchain-javascriptcore`: JavaScriptCore embedding
- `o-toolchain-spidermonkey`: SpiderMonkey embedding
- `o-toolchain-v8`: V8 embedding
- `www`: documentation site

## Configuration

The CLI reads the default engine from:

```toml
# ~/.config/o-/config.toml
[toolchain]
name = "spidermonkey"
```

Built-in engine names:

- `spidermonkey`
- `v8`
- `javascriptcore` or `jsc` on macOS

## CLI

```bash
o- run <path>
o- install
o- install --global <package>
o- uninstall <package>
o- toolchain add <user> <repo>
o- toolchain remove <toolchain>
```

## Running a Script

```bash
cargo run -- run index.js
```

## Package Manager

### Local install

Inside a project with a `package.json`:

```bash
o- install
```

This installs into:

```text
<project>/node_modules
<project>/package-lock.json
```

### Global install

```bash
o- install --global vite
o- install --global eslint@^9
```

Global packages are stored in:

```text
~/.config/o-/packages/node_modules
~/.config/o-/packages/package-lock.json
```

Generated shims are stored in:

```text
~/.config/o-/packages/node_modules/.bin
```

### Uninstall

```bash
o- uninstall vite
```

Uninstall removes:

- the installed package directory
- generated `.bin` shims
- the matching entry in the global lockfile

## Current Package-Manager Scope

Implemented:

- `dependencies`
- `devDependencies`
- `optionalDependencies`
- `peerDependencies`
- tarball download and extraction
- `dist.integrity` verification
- `.bin` shim creation
- global uninstall cleanup

Not implemented yet:

- lifecycle scripts
- workspaces
- hoisting optimization
- `ci` mode
- `package.json` mutation commands

## Documentation

The docs app lives in [`www`](./www) and includes:

- getting started
- engine backend notes
- package manager behavior

## Release Process

This repository is configured for [`release-plz`](https://release-plz.dev/).

- push regular changes to `main`
- `release-plz` opens or updates a release PR
- merging that PR publishes unpublished crates and creates tags/releases

The release workflow expects a `CARGO_REGISTRY_TOKEN` secret with publish
access to crates.io.
