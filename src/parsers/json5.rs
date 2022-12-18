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

use crate::parsers::{FileParserBuilder, Load};
use crate::{AnyResult, Case, Value};
use std::io::Read;

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
        reader.read_to_string(&mut data)?;
        Ok(json5::from_str(&data)?)
    }
}
