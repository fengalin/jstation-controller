# jstation-controller ![CI](https://github.com/fengalin/jstation-controller/workflows/CI/badge.svg)

`jstation-controller` is a cross-platform interface to control the J-Station guitar and bass modeling and effect processing system.

While it is usable to control a J-Station, this application is a work in progress and lacks functionalities. See [gstation-edit](https://github.com/fengalin/gstation-edit) for a full-featured Linux alternative.

![jstation-controller UI](assets/screenshot_20221224.png "jstation-controller UI")

## Features

- [X] Scan the available MIDI ports for a J-Station device.
- [X] Download device Programs.
- [X] Show the parameters for selected Program.
- [X] Use the UI to update a parameter.
- [X] Reflect device parameters updates on the UI.
- [X] Show the list of Programs.
- [X] Change Program from the UI.
- [ ] Rename a Program.
- [ ] Store / undo pending modifications.
- [ ] Import a Program bank from a file.
- [ ] Export a Program bank to a file.

## Dependencies

This application uses the following crates which require system level libraries:

- [`iced`](https://crates.io/crates/iced).
- [`midir`](https://crates.io/crates/midir).

### Linux

Minimum dependencies include development libraries for:

- Wayland (`libwayland-client`, `libwayland-dev`, ...) or X11 (`libxkbcommon-dev`, ...)
- alsa (`alsa-lib-devel`, `libasound2-dev`, ...)

### macOS

If you can test the application on this OS, please open a PR with instructions.

### Windows

If you can test the application on this OS, please open a PR with instructions.

## Build

You need a stable Rust toolchain for the target host. Get it from [this page](https://www.rust-lang.org/fr/tools/install).
On a Unix-like system, you should be able to install `rustup` from your packet
manager.

Clone the git tree and run the following command in an environment where
`cargo` is available:

```
cargo b --release
```

## Run

After a successful compilation, launch the executable with:

```
target/release/jstation-controller
```

## LICENSE

This crate is licensed under MIT license ([LICENSE-MIT](LICENSE-MIT) or
http://opensource.org/licenses/MIT)
