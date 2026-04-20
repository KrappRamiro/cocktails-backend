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

  # worker-build links against OpenSSL at runtime, but devenv does not add the profile's
  # lib dir to LD_LIBRARY_PATH automatically. Without this, worker-build fails at startup
  # with "libssl.so.3: cannot open shared object file". pkgs.openssl.dev (above) only
  # provides headers; pkgs.openssl provides the actual .so runtime libraries.
  env.LD_LIBRARY_PATH = lib.makeLibraryPath [ pkgs.openssl ];

  languages.rust = {
    enable = true;
    channel = "stable";
    targets = ["wasm32-unknown-unknown"];
  };

  claude.code.enable = true;
}
