#!/usr/bin/env bash
# Run every gate, in the order CI runs them. Stops at the first failure.
#
#   scripts/check.sh
set -euo pipefail
cd "$(dirname "$0")/.."

step() { printf '\n==> %s\n' "$*"; }

step "rinf gen (Dart bindings)"
rinf gen

step "cargo clippy"
cargo clippy --all-targets -- -D warnings

step "cargo test"
cargo test

step "flutter analyze"
flutter analyze

step "flutter test"
flutter test

step "versions agree"
scripts/version.sh check

printf '\nAll gates passed.\n'
