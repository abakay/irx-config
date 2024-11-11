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
//! ```
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

use crate::{
    parsers::{FileParserBuilder, Load},
    AnyResult, Case, Value,
};
use std::{
    borrow::Cow,
    io::{Error as IoError, Read},
};
use toml::Value as TomlValue;

/// All errors for `TOML` parser.
#[non_exhaustive]
#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("${1}")]
    IoError(#[source] IoError, Cow<'static, str>),
    #[error("Failed parse TOML")]
    ParseToml(#[source] toml::de::Error),
    #[error("Failed to create value node")]
    Value(#[source] crate::Error),
}

/// Builder for `TOML` parser.
pub type ParserBuilder = FileParserBuilder<LoadToml>;

/// Implements [`Load`] trait for `TOML` parser.
#[derive(Clone, Default)]
pub struct LoadToml;

impl Case for LoadToml {}

impl Load for LoadToml {
    fn load(&mut self, mut reader: impl Read) -> AnyResult<Value> {
        let mut data = String::new();
        reader
            .read_to_string(&mut data)
            .map_err(|e| Error::IoError(e, "Failed read data to buffer".into()))?;
        Ok(Value::try_from(normalize(
            &mut toml::from_str(&data).map_err(Error::ParseToml)?,
        ))
        .map_err(Error::Value)?)
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
