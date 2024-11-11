//! This module provide `YAML` parser implementation.
//!
//! To enable that parser  one has to add the following to Cargo.toml:
//!
//! ```toml
//! [dependencies]
//! irx-config = { version = "2.2", features = ["yaml"] }
//! ```
//!
//! # Example
//!
//! ```
//! use irx_config::ConfigBuilder;
//! use irx_config::parsers::yaml::ParserBuilder;
//!
//! let config = ConfigBuilder::default()
//!     .append_parser(
//!         ParserBuilder::default()
//!             .default_path("config.yaml")
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

/// All errors for `YAML` parser.
#[non_exhaustive]
#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Failed parse YAML")]
    ParseYaml(#[source] serde_yaml::Error),
}

/// Builder for `YAML` parser.
pub type ParserBuilder = FileParserBuilder<LoadYaml>;

/// Implements [`Load`] trait for `YAML` parser.
#[derive(Clone, Default)]
pub struct LoadYaml;

impl Case for LoadYaml {}

impl Load for LoadYaml {
    #[inline]
    fn load(&mut self, reader: impl Read) -> AnyResult<Value> {
        Ok(serde_yaml::from_reader(reader).map_err(Error::ParseYaml)?)
    }
}
