# Coffee Timer

[![CI](https://github.com/lumpsoid/coffee_timer/actions/workflows/ci.yml/badge.svg)](https://github.com/lumpsoid/coffee_timer/actions/workflows/ci.yml)
[![License: 0BSD](https://img.shields.io/badge/license-0BSD-blue.svg)](LICENSE)

A brewing timer for coffee recipes with several steps (bloom, pours,
drawdown, …). Each step counts down on its own, and at every step
boundary the phone vibrates and plays a sound, so you can watch the kettle
instead of the screen. The screen stays awake while you brew.

Everything stays on your phone: no account, no network, no tracking.

## Features

- **Recipes as steps.** Each step has a name, a duration and, optionally,
  how much water to pour. The brew page shows a running "pour to" target
  for the scale.
- **A brew you can steer.** Pause and resume, start from any step, reset.
  A pre-alert buzzes a few seconds before a step ends.
- **Change the dose for one brew.** Swipe the coffee on a gram tape or
  change the ratio; the water follows. Save it to the recipe if you like
  it.
- **Keeps running in the background** on Android, with a notification
  showing the current step.
- **Recipe list as coffee-bag labels.** Pin favourites, reorder, duplicate,
  delete with undo.
- **Import and export** recipes as JSON through the clipboard, to share
  them or move them to another phone.
- **Settings:** light, dark or system theme, accent colour, sound,
  vibration and pre-alert.

## Install

Android: download the APK from the
[latest release](https://github.com/lumpsoid/coffee_timer/releases/latest).
Most phones want `arm64-v8a`; `armeabi-v7a` is for older 32-bit phones and
`x86_64` for emulators. Check the download against `SHA256SUMS`.

The app is written with Flutter, so it can also be built for iOS, Linux,
macOS and Windows from source; only Android is tested and released.

## Build from source

You need [Flutter](https://docs.flutter.dev/get-started/install),
[Rust](https://www.rust-lang.org/tools/install) and the rinf CLI
(`cargo install rinf_cli --version 8.11.0`).

```shell
git clone https://github.com/lumpsoid/coffee_timer.git && cd coffee_timer
flutter pub get
rinf gen      # generates the Dart bindings for the Rust core
flutter run
```

## How it's built

A **Rust core** holds every rule and all state; a **Flutter GUI** renders
what the core sends and passes user taps back. They talk through
[rinf](https://github.com/cunarist/rinf).

```
native/core    rules + state, no I/O        ◀── cargo test
native/store   SQLite persistence
native/hub     rinf bridge to Flutter        native/cli   the same core on the command line
lib/           Flutter GUI
```

The core runs without Flutter: the CLI brews a recipe in the terminal.

```shell
cargo run -p coffee_timer_cli -- --db /tmp/ct.db list
cargo run -p coffee_timer_cli -- --db /tmp/ct.db brew pourover-uuid --fast
```

More in [docs/](docs/README.md): what the app does in detail, the
[architecture](docs/architecture.md), a [module guide](docs/modules.md),
and [maintaining](docs/maintaining.md) (releases, versions, signing).

## Contributing

Ideas, bug reports and pull requests are welcome. Please open an issue
first to talk an idea through before building it; see
[CONTRIBUTING.md](CONTRIBUTING.md).

## License

[0BSD](LICENSE): use it for anything, no conditions. The bundled fonts,
Oswald and Fraunces, are under the SIL Open Font License
([`assets/fonts/`](assets/fonts)). The sounds that
[`scripts/gen-sounds.py`](scripts/gen-sounds.py) generates are 0BSD like
the code.
