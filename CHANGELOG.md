# ChangeLog

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
