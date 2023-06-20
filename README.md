# Pie Chart

[![coverage](https://shields.io/endpoint?url=https://raw.githubusercontent.com/jlyonsmith/pie_chart/main/coverage.json)](https://github.com/jlyonsmith/pie_chart/blob/main/coverage.json)
[![Crates.io](https://img.shields.io/crates/v/pie_chart.svg)](https://crates.io/crates/pie_chart)
[![Docs.rs](https://docs.rs/pie_chart/badge.svg)](https://docs.rs/pie_chart)

A pie chart generator

- [x] Based off of `cargo new`
- [x] Prompts for [Cargo.toml](./Cargo.toml) details
- [x] Uses [`clap`](https://crates.io/crates/clap) for command line processing
- [x] Uses [`colored`](https://crates.io/crates/colored) for messages
- [x] Includes [`lazy_static`](https://crates.io/crates/lazy_static) just because
- [x] Adds a [`Justfile`](https://crates.io/crates/just) with shortcuts for releasing
- [x] Includes a [`version.json5`](./version.json5) file for use with [StampVer](https://crates.io/crates/stampver)
- [x] Includes UNLICENSE file
- [x] Adds [`coverage.json`](./coverage.json) and badge
- [x] README with [crates.io](https://crates.io/) and [docs.rs](https://docs.rs/) badges
- [x] Includes a `scratch` directory
- [x] Includes an [`.vscode/launch.json`](.vscode/launch.json) and other settings
- [x] Includes [`.gitignore`](./.gitignore)
- [x] Creates a named binary file
- [x] Default [`log_macros.rs`](./src/log_macros.rs) for logging
- [x] [`rust-toolchain.toml`](./rust-toolchain.toml) set to `stable`
- [x] Uses a [Deno](https://deno.land/) based [`customize.ts`](./customize.ts) file
- [x] Creates a `.vscode/launch.json` file for debugging
- [x] Includes basic unit tests
- [x] Includes [Criterion](https://crates.io/crates/criterion) and basic benchmark tests
