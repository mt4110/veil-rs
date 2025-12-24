#!/usr/bin/env bash
# DEPRECATED: Wrapper only. Delegates to Cockpit (Nix + Go).
exec nix run .#gen -- "$@"
