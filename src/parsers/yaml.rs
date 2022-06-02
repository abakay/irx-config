//! This module provide `YAML` parser implementation.
//!
//! To enable that parser  one has to add the following to Cargo.toml:
//!
//! ```toml
//! [dependencies]
//! irx-config = { version = "2.1", features = ["yaml"] }
//! ```
//!
//! # Example
//!
//! ```no_run
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

use crate::parsers::{FileParserBuilder, Load};
use crate::{AnyResult, Case, Value};
use std::io::Read;

/// Implements [`Load`] trait for `YAML` parser.
#[derive(Clone)]
pub struct LoadYaml;

impl Case for LoadYaml {}

impl Load for LoadYaml {
    #[inline]
    fn load(&mut self, reader: impl Read) -> AnyResult<Value> {
        Ok(serde_yaml::from_reader(reader)?)
    }
}

/// Builder for `YAML` parser.
pub struct ParserBuilder;

impl ParserBuilder {
    /// Construct instance of `YAML` builder parser.
    pub fn default() -> FileParserBuilder<LoadYaml> {
        let mut builder = FileParserBuilder::default();
        builder.loader(LoadYaml);
        builder
    }
}
