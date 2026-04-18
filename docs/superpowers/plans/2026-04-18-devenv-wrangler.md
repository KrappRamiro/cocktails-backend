# devenv Wrangler Configuration Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Configure the `worker/` devenv shell so that `wrangler`, `worker-build`, and the Rust WASM toolchain are available on PATH without manual installation.

**Architecture:** Add `pkgs.wrangler` and `pkgs.worker-build` to `devenv.nix` packages, enable `languages.rust` with the `wasm32-unknown-unknown` target, and simplify the `wrangler.toml` build command to drop the redundant `cargo install` step.

**Tech Stack:** Nix/devenv, Rust, Cloudflare Workers (`worker-rs`), wrangler CLI

---

### Task 1: Update `devenv.nix`

**Files:**
- Modify: `devenv.nix`

- [ ] **Step 1: Open `devenv.nix` and replace its contents**

Replace the entire file with:

```nix
{
  pkgs,
  lib,
  config,
  inputs,
  ...
}: {
  packages = [
    pkgs.git
    pkgs.wrangler
    pkgs.worker-build
  ];

  languages.rust = {
    enable = true;
    targets = ["wasm32-unknown-unknown"];
  };

  claude.code.enable = true;
}
```

- [ ] **Step 2: Verify the shell loads correctly**

Run:
```bash
devenv shell -- wrangler --version
```

Expected: prints something like `wrangler 4.54.0` with no errors.

- [ ] **Step 3: Verify worker-build is on PATH**

Run:
```bash
devenv shell -- worker-build --version
```

Expected: prints the worker-build version (e.g. `worker-build 0.7.2`) with no errors.

- [ ] **Step 4: Verify the WASM target is installed**

Run:
```bash
devenv shell -- rustup target list --installed
```

Expected: output includes `wasm32-unknown-unknown`.

- [ ] **Step 5: Commit**

```bash
git add devenv.nix
git commit -m "feat: configure devenv for Rust Cloudflare Worker development"
```

---

### Task 2: Simplify `wrangler.toml` build command

**Files:**
- Modify: `wrangler.toml`

- [ ] **Step 1: Update the build command**

In `wrangler.toml`, change:

```toml
[build]
command = "cargo install -q worker-build && worker-build --release"
```

to:

```toml
[build]
command = "worker-build --release"
```

- [ ] **Step 2: Verify the build works end-to-end**

Run (inside devenv shell):
```bash
devenv shell -- wrangler dev --local
```

Expected: wrangler starts a local dev server on `http://localhost:8787` without errors. Ctrl-C to stop.

- [ ] **Step 3: Commit**

```bash
git add wrangler.toml
git commit -m "chore: drop cargo install from wrangler build command (worker-build now on PATH)"
```
