# ChangeLog

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
