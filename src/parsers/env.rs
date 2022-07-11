//! This module provide environment variables parser implementation.
//!
//! The value of each environment variable parsed will be typed according to `YAML` format.
//!
//! To enable that parser  one has to add the following to Cargo.toml:
//!
//! ```toml
//! [dependencies]
//! irx-config = { version = "2.2", features = ["env"] }
//! ```
//!
//! # Example
//!
//! ```no_run
//! use irx_config::ConfigBuilder;
//! use irx_config::parsers::env::ParserBuilder;
//!
//! let config = ConfigBuilder::default()
//!     .append_parser(
//!         ParserBuilder::default()
//!             .default_prefix("APP_")
//!             .prefix_option("prefix")
//!             .build()?,
//!     )
//!     .load()?;
//! ```

use crate::parsers::CowString;
use crate::{AnyResult, Case, Parse, Value, DEFAULT_KEYS_SEPARATOR};
use derive_builder::Builder;
use serde_yaml::Value as YamlValue;
use std::env;

/// The environment variable parser implementation.
#[derive(Builder, Default)]
#[builder(setter(into, strip_option), default)]
pub struct Parser {
    /// Set default prefix for environment variables to be parsed.
    default_prefix: String,
    /// Set prefix option name which could be used to get prefix value from previous parsing [`Value`] results.
    prefix_option: Option<String>,
    /// Set delimiter used to separate keys levels in prefix value. Default is [`DEFAULT_KEYS_SEPARATOR`].
    #[builder(default = "DEFAULT_KEYS_SEPARATOR.to_string()")]
    keys_delimiter: String,
    /// Set delimiter used to separate keys levels in environment variables names. Default is `__`.
    #[builder(default = "\"__\".to_string()")]
    env_keys_delimiter: String,
    /// Set parser's case sensitivity for key names.
    case_sensitive: bool,
    #[builder(setter(skip))]
    value: Option<Value>,
}

impl Case for Parser {
    #[inline]
    fn is_case_sensitive(&self) -> bool {
        self.case_sensitive
    }
}

impl Parse for Parser {
    fn parse(&mut self, value: &Value) -> AnyResult<Value> {
        if let Some(ref v) = self.value {
            return Ok(v.clone());
        }

        let prefix = if let Some(ref p) = self.prefix_option {
            value.get_by_key_path_with_delim(p, &self.keys_delimiter)?
        } else {
            None
        }
        .unwrap_or(CowString::Borrowed(&self.default_prefix));

        let case_on = self.is_case_sensitive();
        let prefix = crate::normalize_case(&prefix, case_on);

        let mut result = Value::with_case(case_on);
        for (k, v) in env::vars_os().filter_map(|(k, v)| {
            let k = k.to_string_lossy();
            let norm_key = crate::normalize_case(&k, case_on);
            if !norm_key.starts_with(prefix.as_ref()) {
                return None;
            }
            Some((norm_key.into_owned(), v.to_string_lossy().to_string()))
        }) {
            let path = k.trim_start_matches(prefix.as_ref());
            let val: YamlValue = serde_yaml::from_str(&v)?;
            result.set_by_key_path_with_delim(path, &self.env_keys_delimiter, val)?;
        }

        self.value = Some(result.clone());
        Ok(result)
    }
}
