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

        # Cockpit Go Binary
        cockpitPkg = pkgs.buildGoModule {
          pname = "cockpit";
          version = "0.0.1";
          src = ./.;
          subPackages = [ "cmd/cockpit" ];
          vendorHash = null;
          # Ensure git is available if tests run during build, or if needed at runtime (but runtime deps are separate)
          # buildGoModule defaults to using `go` from pkgs
        };

        veilApp = {
          type = "app";
          program = "${veilPkg}/bin/veil";
        };

        checkScript = pkgs.writeShellApplication {
          name = "check";
          runtimeInputs = [ pkgs.go_1_24 pkgs.git ];
          text = ''
            unset GOROOT
            unset GOPATH
            GOCACHE=$(mktemp -d)
            export GOCACHE
            trap 'rm -rf "$GOCACHE"' EXIT
            echo "Running check with $(go version)"
            go run ./cmd/check
          '';
        };

        goTestScript = pkgs.writeShellApplication {
          name = "go-test";
          runtimeInputs = [ pkgs.go_1_24 pkgs.git ];
          text = ''
            unset GOROOT
            unset GOPATH
            GOCACHE=$(mktemp -d)
            export GOCACHE
            trap 'rm -rf "$GOCACHE"' EXIT
            echo "Running go-test with $(go version)"
            go test ./...
          '';
        };

        # Phase 10: Cockpit Single Entry apps (Nix-first)
        aiPackScript = pkgs.writeShellApplication {
          name = "ai-pack";
          runtimeInputs = [ pkgs.git ];
          text = ''
            exec ${cockpitPkg}/bin/cockpit ai-pack "$@"
          '';
        };

        genScript = pkgs.writeShellApplication {
          name = "gen";
          runtimeInputs = [ pkgs.git ];
          text = ''
            exec ${cockpitPkg}/bin/cockpit gen "$@"
          '';
        };

        statusScript = pkgs.writeShellApplication {
          name = "status";
          runtimeInputs = [ pkgs.git ];
          text = ''
            exec ${cockpitPkg}/bin/cockpit status "$@"
          '';
        };
      in
      {
        packages.veil = veilPkg;
        packages.cockpit = cockpitPkg;
        packages.default = veilPkg;

        apps.veil = veilApp;
        apps.check = { type = "app"; program = "${checkScript}/bin/check"; };
        apps."go-test" = { type = "app"; program = "${goTestScript}/bin/go-test"; };
        apps."ai-pack" = { type = "app"; program = "${aiPackScript}/bin/ai-pack"; };
        apps.gen = { type = "app"; program = "${genScript}/bin/gen"; };
        apps.status = { type = "app"; program = "${statusScript}/bin/status"; };
        apps.default = veilApp;

        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs; [
            rustStable
            pkg-config
            openssl
            go_1_24

            # Tools
            git
            cargo-edit
            cargo-watch
            cargo-audit
            pre-commit
            nixd
            nixpkgs-fmt

            # Database
            postgresql
            sqlx-cli

            # Automation
            gh
            scorecard
          ];

          shellHook = ''
            unset GOROOT
            unset GOPATH
            export GOTOOLCHAIN=local

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
            git
            cargo-edit
            cargo-watch
            cargo-audit
            pre-commit
            nixd
            nixpkgs-fmt

            # Database
            postgresql
            sqlx-cli

            # Automation
            gh
            scorecard
          ];

          shellHook = ''
            unset GOROOT
            unset GOPATH
            export GOTOOLCHAIN=local

            export PATH="${rustMsrv}/libexec:$PATH"
            export RUST_BACKTRACE=1
            echo "veil-rs dev env loaded (MSRV 1.82.0)" >&2
            echo "Rust version: $(rustc --version)" >&2
          '';
        };
      }
    );
}
