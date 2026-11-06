# Maintaining

What to do, and in what order, for the recurring jobs: a fresh checkout,
everyday work, a release, dependency updates. The scripts live in
[`scripts/`](../scripts).

| Script | Does |
|---|---|
| `scripts/check.sh` | Every gate in CI's order: `rinf gen`, `cargo fmt --check`, clippy, `cargo test`, `flutter analyze`, `flutter test`, versions agree |
| `scripts/version.sh show\|check\|bump X.Y.Z` | Print, check or bump the version in `Cargo.toml` and `pubspec.yaml` |
| `scripts/release.sh` | Check the tree is releasable, run the gates, tag `vX.Y.Z` |
| `scripts/gen-sounds.py` | Regenerate the brew sounds in `assets/sounds/` (needs numpy and `lame`) |

## Fresh checkout

```shell
flutter pub get
rinf gen           # the Dart bindings are gitignored
scripts/check.sh
git task init && git task sync   # only if you use the git-task backlog
```

## Everyday

1. Branch off `main` (`feat/…`, `fix/…`).
2. Change, then run `scripts/check.sh`. Run `rinf gen` after touching
   `native/hub/src/signals/`.
3. Add a line under `## [Unreleased]` in [CHANGELOG.md](../CHANGELOG.md)
   for anything a user would notice.
4. Merge into `main`. CI runs the gates on every push and pull request.

## Version

The version lives in two files that must agree:

- `Cargo.toml` `[workspace.package] version`: the Rust core reads it, and
  the app shows it with the commit, `Version 0.1.0 (2391c1d)`, at the
  foot of Settings (`native/core/build.rs`).
- `pubspec.yaml` `version: X.Y.Z+N`: Android's `versionName` and, as
  `N`, its `versionCode`. `N` must grow with every release, or Android
  refuses the update.

`scripts/version.sh bump X.Y.Z` sets both, raises `N` by one and dates the
changelog's Unreleased section. Before 1.0.0: a minor bump (0.2.0) for
new features or changed behaviour, a patch bump (0.1.1) for fixes.

## Release

1. On `main`, clean tree:
   ```shell
   scripts/version.sh bump 0.2.0
   ```
2. Review `CHANGELOG.md`: the new `## [0.2.0] - date` section is the
   release notes. Commit:
   ```shell
   git commit -am "chore(release): 0.2.0"
   ```
3. Tag (it re-runs every gate first):
   ```shell
   scripts/release.sh
   ```
4. Push the commit and the tag:
   ```shell
   git push origin main v0.2.0
   ```
5. The [Release workflow](../.github/workflows/release.yml) builds the
   signed APKs (one per ABI, with `SHA256SUMS`) and publishes a GitHub
   Release with the changelog section as its notes. Check it on the
   repository's Releases page.

For the first release, step 1 is `scripts/version.sh bump 0.1.0`: the
version stays 0.1.0, the build number goes to 2, and Unreleased becomes
`[0.1.0]`.

If a release goes wrong, delete the tag (`git push origin :v0.2.0`,
`git tag -d v0.2.0`) and its GitHub Release, fix, and tag again.
Never reuse a build number that reached a device.

## Android signing key

A release is signed with one key for the app's lifetime. **If the key is
lost, installed apps cannot be updated**; users would have to uninstall
(losing their recipes unless exported). Keep the keystore and its
passwords in a password manager, with a backup.

Create it once:

```shell
keytool -genkeypair -v -keystore coffee-timer-release.jks \
  -alias coffee-timer -keyalg RSA -keysize 4096 -validity 36500
```

For local release builds, put the keystore at `android/release.jks` and
write `android/key.properties` (both gitignored):

```properties
storeFile=release.jks
storePassword=…
keyAlias=coffee-timer
keyPassword=…
```

Without `key.properties`, release builds fall back to the debug key, which
is fine for `flutter run --release` but must never be published.

For CI, add these repository secrets (Settings → Secrets and variables →
Actions):

| Secret | Value |
|---|---|
| `ANDROID_KEYSTORE_BASE64` | `base64 -w0 coffee-timer-release.jks` |
| `ANDROID_KEYSTORE_PASSWORD` | the store password |
| `ANDROID_KEY_ALIAS` | `coffee-timer` |
| `ANDROID_KEY_PASSWORD` | the key password |

## Dependencies

Dependabot opens weekly pull requests for cargo, pub, Gradle and GitHub
Actions, minor and patch bumps grouped per ecosystem. Merge them when CI
is green; read the changelog of a major bump first.

Some updates need a hand:

- **rinf** is three things that must match: the `rinf` crate
  (`native/hub/Cargo.toml`), the `rinf` Dart package (`pubspec.yaml`) and
  the rinf CLI (`RINF_CLI_VERSION` in both workflows, and your local
  `cargo install rinf_cli --version …`). Bump all of them together, run
  `rinf gen`, then the gates. Dependabot bumps the first two separately;
  hold them until all three move.
- **Flutter** follows the stable channel in CI. After upgrading locally
  (`flutter upgrade`), run the gates and a real build
  (`flutter build apk --release`).
- **Gradle, AGP, Kotlin** (`android/settings.gradle.kts`,
  `android/gradle/wrapper/gradle-wrapper.properties`): check with
  `flutter build apk --release`, which CI does not run until a release.
- **Rust** is the stable toolchain; `rustup update`, then the gates.

## GitHub repository settings

Once, on GitHub:

- Description and topics (`flutter`, `rust`, `rinf`, `coffee`,
  `brewing`, `timer`, `android`).
- Settings → General → Features: Issues on.
- Settings → Code security: Dependabot alerts and security updates on.
- Settings → Branches: optionally protect `main` and require the CI
  checks.
