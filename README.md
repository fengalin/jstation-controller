# jstation-controller ![CI](https://github.com/fengalin/jstation-controller/workflows/CI/badge.svg)

`jstation-controller` aims to be a cross-platform interface to control the J-Station
guitare and bass modeling and effect processing system.

Note that this application is in early development. See [gstation-edit](https://github.com/fengalin/gstation-edit) for a working Linux alternative.

## Dependencies

This application uses the following crates which require system level libraries:

- [`iced`](https://crates.io/crates/iced).
- [`midir`](https://crates.io/crates/midir).

### Linux

Minimum dependencies include development libraries for:

- X11 or Wayland.
- `alsa` (`alsa-lib-devel`, `libasound2-dev`, ...)

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
