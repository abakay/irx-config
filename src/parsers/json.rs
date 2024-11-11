//! This module provide `JSON` parser implementation.
//!
//! To enable that parser  one has to add the following to Cargo.toml:
//!
//! ```toml
//! [dependencies]
//! irx-config = { version = "2.2", features = ["json"] }
//! ```
//!
//! # Example
//!
//! ```
//! use irx_config::ConfigBuilder;
//! use irx_config::parsers::json::ParserBuilder;
//!
//! let config = ConfigBuilder::default()
//!     .append_parser(
//!         ParserBuilder::default()
//!             .default_path("config.json")
//!             .path_option("config")
//!             .build()?,
//!     )
//!     .load()?;
//! ```

use crate::{
    parsers::{FileParserBuilder, Load},
    AnyResult, Case, Value,
};
use std::io::Read;

/// All errors for `JSON` parser.
#[non_exhaustive]
#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Failed parse JSON")]
    ParseJson(#[source] serde_json::Error),
}

/// Builder for `JSON` parser.
pub type ParserBuilder = FileParserBuilder<LoadJson>;

/// Implements [`Load`] trait for `JSON` parser.
#[derive(Clone, Default)]
pub struct LoadJson;

impl Case for LoadJson {}

impl Load for LoadJson {
    #[inline]
    fn load(&mut self, reader: impl Read) -> AnyResult<Value> {
        Ok(serde_json::from_reader(reader).map_err(Error::ParseJson)?)
    }
}
