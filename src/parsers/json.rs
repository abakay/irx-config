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
//! ```no_run
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

use crate::parsers::{FileParserBuilder, Load};
use crate::{AnyResult, Case, Value};
use std::io::Read;

/// Builder for `JSON` parser.
pub type ParserBuilder = FileParserBuilder<LoadJson>;

/// Implements [`Load`] trait for `JSON` parser.
#[derive(Clone, Default)]
pub struct LoadJson;

impl Case for LoadJson {}

impl Load for LoadJson {
    #[inline]
    fn load(&mut self, reader: impl Read) -> AnyResult<Value> {
        Ok(serde_json::from_reader(reader)?)
    }
}
