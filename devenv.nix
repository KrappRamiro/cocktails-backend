{
  pkgs,
  lib,
  config,
  inputs,
  ...
}: {
  # worker-build is intentionally absent: nixpkgs ships 0.7.x which is incompatible
  # with worker-rs 0.8. wrangler.toml pins `cargo install worker-build@^0.8` instead.
  # openssl.dev + pkg-config are required for `cargo install worker-build` to compile.
  packages = [
    pkgs.git
    pkgs.wrangler
    pkgs.openssl.dev
    pkgs.pkg-config
  ];

  languages.rust = {
    enable = true;
    channel = "stable";
    targets = ["wasm32-unknown-unknown"];
  };

  claude.code.enable = true;
}
