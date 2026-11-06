#!/usr/bin/env bash
# Tag the current commit as a release, after checking it is releasable.
# Pushing the tag makes CI build the APK and publish a GitHub Release.
#
#   scripts/release.sh            tag vX.Y.Z from the version in the tree
#
# Run scripts/version.sh bump X.Y.Z and commit it first. This script
# does not push; it prints the command.
set -euo pipefail
cd "$(dirname "$0")/.."

fail() { echo "release: $*" >&2; exit 1; }

[[ "$(git branch --show-current)" == main ]] || fail "not on main"
[[ -z "$(git status --porcelain)" ]] || fail "working tree is not clean"

scripts/version.sh check >/dev/null
version=$(scripts/version.sh show | cut -d+ -f1)
tag="v$version"

git rev-parse -q --verify "refs/tags/$tag" >/dev/null && fail "$tag already exists"
grep -q "^## \[$version\]" CHANGELOG.md || fail "CHANGELOG.md has no [$version] section"

scripts/check.sh

# The changelog section becomes the tag message (and the release notes).
notes=$(awk -v v="$version" '
  $0 ~ "^## \\[" v "\\]" { on = 1; next }
  on && /^## \[/ { exit }
  on { print }
' CHANGELOG.md)

# verbatim: the default cleanup would drop the "### Added" headings as comments.
git tag -a --cleanup=verbatim "$tag" -m "$tag" -m "$notes"
echo
echo "Tagged $tag. Publish it with:"
echo "  git push origin main $tag"
