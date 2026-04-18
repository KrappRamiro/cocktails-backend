# devenv Wrangler Configuration Design

## Goal

Configure the `worker/` devenv shell for Rust Cloudflare Worker development with `wrangler` and `worker-build` available on PATH.

## Changes

### `devenv.nix`

- Enable `languages.rust` with the `wasm32-unknown-unknown` target (required for Cloudflare Workers)
- Add `pkgs.wrangler` (CLI for `wrangler dev`, `wrangler deploy`, etc.)
- Add `pkgs.worker-build` (Rust → WASM build tool for `workers-rs` projects)

### `wrangler.toml`

- Simplify the build command from `cargo install -q worker-build && worker-build --release` to `worker-build --release`, since `worker-build` is now provided by the devenv shell.

## What's not included

- No auto-start of `wrangler dev` as a devenv process — it's run manually.
- No `rust-toolchain.toml` file — toolchain is declared inline in `devenv.nix`.
