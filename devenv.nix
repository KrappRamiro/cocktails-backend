{
  pkgs,
  lib,
  config,
  inputs,
  ...
}: {
  # worker-build is intentionally absent: nixpkgs ships 0.7.x which is incompatible
  # with worker-rs 0.8. wrangler.toml pins `cargo install worker-build@^0.8` instead.
  packages = [
    pkgs.git
    pkgs.wrangler
  ];

  languages.rust = {
    enable = true;
    channel = "stable";
    targets = ["wasm32-unknown-unknown"];
  };

  claude.code.enable = true;
}
