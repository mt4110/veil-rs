{
  description = "veil-rs development environment";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { nixpkgs, rust-overlay, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs { inherit system overlays; };

        # devShell 用の Nightly
        rustToolchain =
          pkgs.rust-bin.selectLatestNightlyWith (toolchain:
            toolchain.default.override {
              extensions = [ "rust-src" "rust-analyzer" "clippy" ];
            });

        # ★ ここが新規：veil バイナリパッケージ
        veilPkg = pkgs.rustPlatform.buildRustPackage {
          pname = "veil";
          version = "0.8.0";
          src = ./.;

          cargoLock.lockFile = ./Cargo.lock;

          # ワークスペースの中の CLI crate を明示
          cargoBuildFlags = [ "--package" "veil-cli" "--bin" "veil" ];

          # OpenSSL とか必要ならここで
          nativeBuildInputs = [ pkgs.pkg-config ];
          buildInputs = [ pkgs.openssl ];
        };

        veilApp = {
          type = "app";
          program = "${veilPkg}/bin/veil";
        };
      in
      {
        packages.veil = veilPkg;
        packages.default = veilPkg;

        apps.veil = veilApp;
        apps.default = veilApp;

        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs; [
            rustToolchain
            pkg-config
            openssl

            # Tools
            cargo-edit
            cargo-watch
            cargo-audit
            pre-commit
            nixd
            nixpkgs-fmt

            # Database
            postgresql
            sqlx-cli
          ];

          shellHook = ''
            export PATH="${rustToolchain}/libexec:$PATH"
            export RUST_BACKTRACE=1
            echo "🔮 veil-rs development environment loaded (NIGHTLY)!" >&2
            echo "Rust version: $(rustc --version)" >&2
          '';
        };
      }
    );
}
