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

        # --- toolchains ---
        rustStable =
          pkgs.rust-bin.stable.latest.default.override {
            extensions = [ "rust-src" "rust-analyzer" "clippy" "rustfmt" ];
          };

        rustMsrv =
          pkgs.rust-bin.stable."1.82.0".default.override {
            extensions = [ "rust-src" "rust-analyzer" "clippy" ];
          };

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
            rustStable
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
            export PATH="${rustStable}/libexec:$PATH"
            export RUST_BACKTRACE=1
            echo "veil-rs dev env loaded (stable)" >&2
            echo "Rust version: $(rustc --version)" >&2
          '';
        };

        devShells.msrv = pkgs.mkShell {
          buildInputs = with pkgs; [
            rustMsrv
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
            export PATH="${rustMsrv}/libexec:$PATH"
            export RUST_BACKTRACE=1
            echo "veil-rs dev env loaded (MSRV 1.82.0)" >&2
            echo "Rust version: $(rustc --version)" >&2
          '';
        };
      }
    );
}
