//! This module provide `JSON5` parser implementation.
//!
//! To enable that parser  one has to add the following to Cargo.toml:
//!
//! ```toml
//! [dependencies]
//! irx-config = { version = "2.2", features = ["json5-parser"] }
//! ```
//!
//! # Example
//!
//! ```no_run
//! use irx_config::ConfigBuilder;
//! use irx_config::parsers::json5::ParserBuilder;
//!
//! let config = ConfigBuilder::default()
//!     .append_parser(
//!         ParserBuilder::default()
//!             .default_path("config.json5")
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

/// All errors for `JSON5` parser.
#[non_exhaustive]
#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("${1}")]
    IoError(#[source] IoError, Cow<'static, str>),
    #[error("Failed parse JSON5")]
    ParseJson5(#[source] json5::Error),
}

/// Builder for `JSON5` parser.
pub type ParserBuilder = FileParserBuilder<LoadJson>;

/// Implements [`Load`] trait for `JSON5` parser.
#[derive(Clone, Default)]
pub struct LoadJson;

impl Case for LoadJson {}

impl Load for LoadJson {
    #[inline]
    fn load(&mut self, mut reader: impl Read) -> AnyResult<Value> {
        let mut data = String::new();
        reader
            .read_to_string(&mut data)
            .map_err(|e| Error::IoError(e, "Failed read data to buffer".into()))?;
        Ok(json5::from_str(&data).map_err(Error::ParseJson5)?)
    }
}
