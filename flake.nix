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
            extensions = [ "rust-src" "rust-analyzer" "clippy" "rustfmt" ];
          };

        # --- veil (Rust) ---
        veilPkg = pkgs.rustPlatform.buildRustPackage {
          pname = "veil";
          version = "0.8.0";
          src = ./.;
          cargoLock.lockFile = ./Cargo.lock;
          cargoBuildFlags = [ "--package" "veil-cli" "--bin" "veil" ];
          nativeBuildInputs = [ pkgs.pkg-config ];
          buildInputs = [ pkgs.openssl ];
        };

        veilApp = {
          type = "app";
          program = "${veilPkg}/bin/veil";
        };

        # --- veil-aiux (Go | Phase 9 Cockpit) ---
        veilAiuxPkg = pkgs.buildGoModule {
          pname = "veil-aiux";
          version = "0.1.0";
          src = ./tools/veil-aiux;
          vendorHash = null; # Todo: update after go.mod populated
          # Force git dependency if needed
          nativeBuildInputs = [ pkgs.git ];
        };

        # Automation Apps (Wrappers for proper subcommand dispatch)
        scriptGen = pkgs.writeShellScriptBin "gen" ''
          ${veilAiuxPkg}/bin/veil-aiux gen "$@"
        '';
        scriptCheck = pkgs.writeShellScriptBin "check" ''
          ${veilAiuxPkg}/bin/veil-aiux check "$@"
        '';
        scriptStatus = pkgs.writeShellScriptBin "status" ''
          ${veilAiuxPkg}/bin/veil-aiux status "$@"
        '';

      in
      {
        packages.veil = veilPkg;
        packages.veil-aiux = veilAiuxPkg;
        packages.default = veilPkg;

        apps.veil = veilApp;
        apps.default = veilApp;

        # Phase 9 Automation Apps
        apps.gen = { type = "app"; program = "${scriptGen}/bin/gen"; };
        apps.check = { type = "app"; program = "${scriptCheck}/bin/check"; };
        apps.status = { type = "app"; program = "${scriptStatus}/bin/status"; };

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
            
            # Phase 9 Tools
            go
            git

            # Database
            postgresql
            sqlx-cli
          ];

          shellHook = ''
            export PATH="${rustStable}/libexec:$PATH"
            export RUST_BACKTRACE=1
            echo "veil-rs dev env loaded (stable)" >&2
            echo "Phase 9 Automation: veil-aiux (Go) available" >&2
            echo "Rust version: $(rustc --version)" >&2
            echo "Go version: $(go version)" >&2
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

            # Phase 9 Tools
            go
            git

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
