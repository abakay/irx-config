//! This module provide `TOML` parser implementation.
//!
//! To enable that parser  one has to add the following to Cargo.toml:
//!
//! ```toml
//! [dependencies]
//! irx-config = { version = "2.2", features = ["toml-parser"] }
//! ```
//!
//! # Example
//!
//! ```no_run
//! use irx_config::ConfigBuilder;
//! use irx_config::parsers::toml::ParserBuilder;
//!
//! let config = ConfigBuilder::default()
//!     .append_parser(
//!         ParserBuilder::default()
//!             .default_path("config.toml")
//!             .path_option("config")
//!             .build()?,
//!     )
//!     .load()?;
//! ```

use crate::parsers::{FileParserBuilder, Load};
use crate::{AnyResult, Case, Value};
use std::io::Read;
use toml::Value as TomlValue;

/// Builder for `TOML` parser.
pub type ParserBuilder = FileParserBuilder<LoadToml>;

/// Implements [`Load`] trait for `TOML` parser.
#[derive(Clone, Default)]
pub struct LoadToml;

impl Case for LoadToml {}

impl Load for LoadToml {
    fn load(&mut self, mut reader: impl Read) -> AnyResult<Value> {
        let mut data = String::new();
        reader.read_to_string(&mut data)?;
        Ok(Value::try_from(normalize(&mut toml::from_str(&data)?))?)
    }
}

fn normalize(value: &mut TomlValue) -> &mut TomlValue {
    if let TomlValue::Table(map) = value {
        for (_, val) in map.iter_mut() {
            if let TomlValue::Datetime(dt) = val {
                *val = TomlValue::String(dt.to_string());
            } else {
                normalize(val);
            }
        }
    }
    value
}
