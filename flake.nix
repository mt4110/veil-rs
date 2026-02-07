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

          nativeBuildInputs = [ pkgs.makeWrapper ];

          postFixup = ''
            wrapProgram $out/bin/cockpit \
              --prefix PATH : ${pkgs.lib.makeBinPath [ pkgs.git pkgs.scorecard ]}
          '';
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
        prverifyScript = pkgs.writeShellApplication {
          name = "prverify";
          runtimeInputs = [ pkgs.bash pkgs.coreutils pkgs.git pkgs.go_1_24 rustStable ];
          text = ''
              set -u
              set -o pipefail

              root="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
              cd "$root"

              out_dir=".local/prverify"
              mkdir -p "$out_dir"

              ts="$(date -u +%Y%m%dT%H%M%SZ)"
              sha="$(git rev-parse --short HEAD 2>/dev/null || echo no-git)"
              report="$(printf '%s/prverify_%s_%s.md' "$out_dir" "$ts" "$sha")"

              : > "$report"
              exec > >(tee -a "$report") 2>&1

              write_header() {
                cat <<'HDR'
            # PR verify report

            このレポートは `nix run .#prverify` の実行結果です。

            ## Environment
            HDR
                echo ""
                echo "- timestamp (UTC): $ts"
                echo "- git sha: $sha"
                echo "- rustc: $(rustc --version 2>/dev/null || echo 'N/A')"
                echo "- cargo: $(cargo --version 2>/dev/null || echo 'N/A')"
                echo "- go: $(go version 2>/dev/null || echo 'N/A')"
                echo ""
                echo "## Commands"
                echo '```bash'
                echo 'cargo test -p veil-cli --test cli_tests'
                echo 'cargo test --workspace'
                echo 'go run ./cmd/prverify'
                echo '```'
                echo ""
              }

              run_block() {
                local title="$1"; shift
                echo "## $title"
                echo '```'
                "$@" 2>&1
                local rc=$?
                echo '```'
                echo ""
                echo "- exit_code: $rc"
                echo ""
                return $rc
              }

              write_header

              rc1=0
              rc2=0
              rc3=0

              run_block "cli smoke (trycmd)" cargo test -p veil-cli --test cli_tests || rc1=$?
              run_block "workspace tests" cargo test --workspace || rc2=$?

              # Drift Check (Go)
              {
                unset GOROOT
                unset GOPATH
                export GOCACHE
                GOCACHE=$(mktemp -d)
                # Note: trap for cleanup should be handled carefully in concatenated scripts,
                # but for prverify it's fine.
                echo "## Drift Check (Go)"
                echo '```'
                go run ./cmd/prverify "$@" 2>&1
                rc3=$?
                echo '```'
                echo ""
                echo "- exit_code: $rc3"
                echo ""
                rm -rf "$GOCACHE"
              }

              echo "## Notes / Evidence"
              echo ""
              if [ "$rc1" -eq 0 ] && [ "$rc2" -eq 0 ] && [ "$rc3" -eq 0 ]; then
                echo "- Local run: PASS"
              else
                echo "- Local run: FAIL (cli_tests=$rc1, workspace=$rc2, drift=$rc3)"
              fi
              echo ""

              echo "## Rollback"
              echo ""
              echo '```bash'
              echo '# 1コミットだけ戻す'
              echo 'git revert <commit>'
              echo ""
              echo '# 範囲でまとめて戻す'
              echo 'git revert <oldest_commit>^..<newest_commit>'
              echo '```'
              echo ""

              echo "---"
              echo "report: $report"

              if [ "$rc1" -ne 0 ]; then exit "$rc1"; fi
              if [ "$rc2" -ne 0 ]; then exit "$rc2"; fi
              if [ "$rc3" -ne 0 ]; then exit "$rc3"; fi
              exit 0
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
        apps.prverify = { type = "app"; program = "${prverifyScript}/bin/prverify"; };
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
