The `irx-config` library provides convenient way to represent/parse configuration from different sources. The main
goals is to be very easy to use and to be extendable.

## Features

* Fully compatible with [serde](https://serde.rs/)
* Full deep merge of nested dictionaries/mappings
* Case sensitive/insensitive parameters names matching/merging
* Sealing secrets during display/debugging
* Get all configuration parameters or just cherry pick few
* Several embedded parsers available via library features:
  * Command-line argument (via [clap](https://github.com/clap-rs/clap))
  * Environment variables
  * File based parsers: `JSON`, `JSON5`, `YAML` and `TOML`
* Could be extended with custom parsers

## Examples

### JSON with environment variables

To enable parsers used in example below, one has to add the following to `Cargo.toml`:

```toml
[dependencies]
irx-config = { version = "2.2", features = ["env", "json"] }
```

```rust
use irx_config::parsers::{env, json};
use irx_config::ConfigBuilder;
use serde::Deserialize;

#[derive(Deserialize)]
struct Conf {
    id: u32,
    logger: String,
    tag: String,
}

// Data from two parsers will be merged. The values from parser appended first (`JSON`)
// will take precedence if values have a same names
let config = ConfigBuilder::default()
    .append_parser(
        json::ParserBuilder::default()
            .default_path("config.json")
            .build()?,
    )
    .append_parser(
        env::ParserBuilder::default()
            .default_prefix("APP_")
            .build()?,
    )
    .load()?;

let conf_data: Conf = config.get()?;
```

### Command-line, TOML and environment variables

To enable parsers used in example below, one has to add the following to `Cargo.toml`:

```toml
[dependencies]
irx-config = { version = "2.2", features = ["cmd", "env", "toml-parser"] }
```

```rust
use clap::app_from_crate;
use irx_config::parsers::{cmd, env, toml};
use irx_config::ConfigBuilder;
use serde::Deserialize;

#[derive(Deserialize)]
struct Logger {
    level: String,
    path: String,
}

#[derive(Deserialize)]
struct Connection {
    #[serde(default = "localhost")]
    host: String,
    port: u16,
}

#[derive(Deserialize)]
struct Conf {
    id: u32,
    logger: Logger,
    connection: Connection,
}

let app = app_from_crate!();

// Data from three parsers will be merged. The values from parser appended first (`cmd`)
// will take precedence if values have a same names
let config = ConfigBuilder::default()
    .append_parser(
        cmd::ParserBuilder::new(app)
            .exit_on_error(true)
            .build()?,
    )
    .append_parser(
        toml::ParserBuilder::default()
            .default_path("config.toml")
            .path_option("config")
            .build()?,
    )
    .append_parser(
        env::ParserBuilder::default()
            .default_prefix("APP_")
            .prefix_option("prefix")
            .build()?,
    )
    .load()?;

let conf_data: Conf = config.get()?;
```

### Custom parser

```rust
use irx_config::{AnyResult, Case, ConfigBuilder, Parse, Value};
use serde::Deserialize;
use std::borrow::Cow;

#[derive(Deserialize)]
struct Conf {
    id: u32,
    logger: String,
    tag: String,
}

struct JsonStringParser<'a> {
    data: Cow<'a, str>,
}

impl<'a> JsonStringParser<'a> {
    pub fn new(data: impl Into<Cow<'a, str>>) -> Self {
        JsonStringParser { data: data.into() }
    }
}

impl Case for JsonStringParser<'_> {}

impl Parse for JsonStringParser<'_> {
    fn parse(&mut self, _value: &Value) -> AnyResult<Value> {
        Ok(serde_json::from_str(&self.data)?)
    }
}

let data = r#"{ "id": 42, "logger": "file", "tag": "test" }"#;
let config = ConfigBuilder::load_one(JsonStringParser::new(data))?;
let conf_data: Conf = config.get()?;
```

### JSON parser get partial data

To enable parsers used in example below, one has to add the following to `Cargo.toml`:

```toml
[dependencies]
irx-config = { version = "2.2", features = ["json"] }
```

```rust
use irx_config::parsers::json;
use irx_config::ConfigBuilder;
use serde::Deserialize;

#[derive(Deserialize)]
struct Logger {
    level: String,
    path: String,
}

#[derive(Deserialize)]
struct Connection {
    #[serde(default = "localhost")]
    host: String,
    port: u16,
}

let config = ConfigBuilder::load_one(
    json::ParserBuilder::default()
        .default_path("config.json")
        .build()?,
)?;

let logger: Logger = config.get_by_key_path("logger")?.unwrap();
let port: u16 = config.get_by_key_path("connection:port")?.unwrap();
```
