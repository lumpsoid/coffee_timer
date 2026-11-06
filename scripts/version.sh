#!/usr/bin/env bash
# The app's version lives in two places that must agree: the Cargo
# workspace (Cargo.toml, read by the Rust core and shown in the app) and
# pubspec.yaml (Android's versionName, plus the +N build number that is
# Android's versionCode and must grow with every release).
#
#   scripts/version.sh show          print the version, e.g. 0.1.0+1
#   scripts/version.sh check         fail if the two files disagree
#   scripts/version.sh bump X.Y.Z    set both to X.Y.Z, build number + 1,
#                                    and date the CHANGELOG's Unreleased
#                                    section as [X.Y.Z]
set -euo pipefail
cd "$(dirname "$0")/.."

cargo_version() {
  # The first `version = "..."` is [workspace.package]'s.
  sed -n 's/^version = "\(.*\)"$/\1/p' Cargo.toml | head -n 1
}
pubspec_full() { sed -n 's/^version: \(.*\)$/\1/p' pubspec.yaml; }
pubspec_version() { pubspec_full | cut -d+ -f1; }
pubspec_build() { pubspec_full | cut -d+ -f2; }

check() {
  local cargo pub
  cargo=$(cargo_version)
  pub=$(pubspec_version)
  if [[ "$cargo" != "$pub" ]]; then
    echo "version mismatch: Cargo.toml $cargo, pubspec.yaml $pub" >&2
    exit 1
  fi
  echo "versions agree: $cargo"
}

bump() {
  local next=$1
  if ! [[ "$next" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
    echo "not a version: $next (want X.Y.Z)" >&2
    exit 1
  fi
  if ! grep -q '^## \[Unreleased\]' CHANGELOG.md; then
    echo "CHANGELOG.md has no '## [Unreleased]' section to release" >&2
    exit 1
  fi
  local build=$(( $(pubspec_build) + 1 ))
  sed -i "0,/^version = \".*\"$/s//version = \"$next\"/" Cargo.toml
  sed -i "s/^version: .*$/version: $next+$build/" pubspec.yaml
  sed -i "s/^## \[Unreleased\]$/## [Unreleased]\n\n## [$next] - $(date +%F)/" CHANGELOG.md
  # Refresh Cargo.lock's own crate versions, nothing else.
  cargo update --workspace --quiet
  check
  echo "now $next+$build; review CHANGELOG.md, then commit"
}

case "${1:-}" in
  show) pubspec_full ;;
  check) check ;;
  bump) bump "${2:?usage: scripts/version.sh bump X.Y.Z}" ;;
  *)
    sed -n '2,12p' "$0" >&2
    exit 1
    ;;
esac
