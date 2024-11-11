# ChangeLog

## 2024-11-11 -- 3.5.0

Changes:

* Make hash configurable via crate features: `blake2b` and `blake3`. Use `blake2b` feature by default.
* Switch to dependency namespace `dep:...` in `Cargo.toml` file.
* Set `Rust` version to `1.74.0` in `Cargo.toml` file.
* Upgrade [derive_builder](https://docs.rs/derive_builder/) dependency to the next minor version `0.20.x`.
* Upgrade [clap](https://docs.rs/clap/) dependency to the next minor version `4.5.x`.
* Disable doc tests for create's documentation.
* Minor code cleanups.

## 2023-12-11 -- 3.4.0

Changes:

* Switch from `BLAKE2b` to `BLAKE3` hash.
* Upgrade [clap](https://docs.rs/clap/) dependency to the next minor version `4.4.x`.
* Upgrade [toml](https://docs.rs/toml/) dependency to the next minor version `0.8.x`.

## 2023-02-23 -- 3.3.0

Changes:

* Set `Rust` version to `1.65.0` in `Cargo.toml` file.
* Improved error reporting.

## 2023-02-05 -- 3.2.0

Changes:

* Set `Rust` version to `1.64.0` in `Cargo.toml` file.
* Fixed compilation/clippy warnings for `Rust 1.67.0`.
* Upgrade [clap](https://docs.rs/clap/) dependency to the next minor version `4.1.x`.
* Upgrade [toml](https://docs.rs/toml/) dependency to the next minor version `0.7.x`.
* Minor code improvements.

## 2022-12-18 -- 3.1.1

Changes:

* Fixed compilation/clippy warnings for `Rust 1.66.0`.
* Upgrade [derive_builder](https://docs.rs/derive_builder/) dependency to the next minor version `0.12.x`.
* Minor code improvements.

## 2022-10-08 -- 3.1.0

Changes:

* Fixed issue with default values for `clap` command-line parameters.
* Added a new setting `use_defaults` to `cmd` parser. Use defaults from `clap` arguments. Default is `false`.
**IMPORTANT:** Once that setting will be set to `true` then all defined command-line parameters will have values which will override values with same key(s) from parsers which was added to `ConfigBuilder` after this parser.

## 2022-10-06 -- 3.0.0

Changes:

* Upgrade [clap](https://docs.rs/clap/) dependency to the next major version `4.0.x`.
* Remove `single_flags_as_bool` setting from `cmd` parser. That functionality should be achieved via `clap::ArgAction`.
* The default value of `use_arg_types` settings was changed to `true`.

## 2022-09-13 -- 2.4.0

Changes:

* Upgrade [clap](https://docs.rs/clap/) dependency to the next minor version `3.2.x`.
* Upgrade [serde_yaml](https://docs.rs/serde_yaml/) dependency to the next minor version `0.9.x`.
* Upgrade [derive_builder](https://docs.rs/derive_builder/) dependency to the next minor version `0.11.x`.
* Added a new setting `use_arg_types` to `cmd` parser. If that setting is set to `true` then use `ArgAction` or `ValueParser` type to calculate type of an argument. Default is `false`.

## 2022-09-09 -- 2.3.2

Fixed issue for `use_value_delimiter` with single value in `cmd` parser.

## 2022-09-07 -- 2.3.1

Fixed implicit conversion of `crate::Error` to `anyhow::Error`.

## 2022-09-05 -- 2.3.0

Enhancements:

* Added a new setting `single_flags_as_bool` to `cmd` parser. If that setting is set to `true` and a parameter **not allowed to have multiple occurrences** by [clap](https://docs.rs/clap/) API then parameter's value will have boolean `true` as a value. By default all command-line parameters without values will have they values set to number of they occurrences.
* Added a new setting `ignore_missing_file` to file based parsers. If that setting is set to `true` then a file does not exists, do not try to load it. The default `Value` will be returned. Default is `false`.

## 2022-07-07 -- 2.2.0

Added `JSON5` parser via `json5-parser` feature.

## 2022-06-01 -- 2.1.0

Upgrade [clap](https://docs.rs/clap/) dependency to the next minor version `3.1.x`.

## 2022-01-23 -- 2.0.0

**NOTE:** The [clap](https://docs.rs/clap/) was upgraded to next major version `3.x`. Some old `API` was deprecated (`clap::load_yaml!`, etc.). These changes also forced an increase in the major version for this crate.

* Upgrade [clap](https://docs.rs/clap/) dependency to the next major version `3.x`.
* The following methods was removed from `cmd::ParserBuilder`:
  * `default()`, replaced by `new(...)`
  * `matches(...)`
  * `arg_names(...)`
  * `try_arg_names_from_yaml(...)`

## 2021-12-31 -- 1.0.2

* Fixed compilation warnings for `Rust 1.57.0`.
* Minor code improvements.

## 2021-12-27 -- 1.0.1

Generate documentation on [docs.rs](https://docs.rs/) for all features.

## 2021-12-27 -- 1.0.0

The `irx-config` library provides convenient way to represent/parse configuration from different sources. The main
goals is to be very easy to use and to be extendable.

### Features

* Fully compatible with [serde](https://serde.rs/)
* Full deep merge of nested dictionaries/mappings
* Case sensitive/insensitive parameters names matching/merging
* Sealing secrets during display/debugging
* Get all configuration parameters or just cherry pick few
* Several embedded parsers available via library features:
  * Command-line argument (via [clap](https://github.com/clap-rs/clap))
  * Environment variables
  * File based parsers: `JSON`, `YAML` and `TOML`
* Could be extended with custom parsers
