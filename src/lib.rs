#![doc = include_str!("../README.md")]

pub mod config;
#[cfg(feature = "parsers")]
pub mod parsers;
#[cfg(test)]
mod tests;
pub mod value;

use crate::value::SerdeError;
pub use crate::{
    config::{Config, ConfigBuilder},
    value::{json, Value},
};
use std::{
    borrow::Cow, error::Error as StdError, fmt::Debug, io::Error as IoError,
    result::Result as StdResult,
};

/// A result type with internal error.
pub type Result<T> = StdResult<T, Error>;

type AnyError = Box<dyn StdError + Send + Sync + 'static>;

/// A result type with any error.
pub type AnyResult<T> = StdResult<T, AnyError>;

type CowString<'a> = Cow<'a, str>;
type AnyParser = Box<dyn Parse>;

/// Default key level separator.
pub const DEFAULT_KEYS_SEPARATOR: &str = ":";

/// Error generated during any crate operations.
#[non_exhaustive]
#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Failed to {0} path: '{1}' with empty key path separator")]
    EmptySeparator(&'static str, String),
    #[error("Mapping object expected")]
    NotMap,
    #[error("{1}")]
    SerdeError(#[source] SerdeError, Cow<'static, str>),
    #[error("Failed to parse value for parser #{1}")]
    ParseValue(#[source] AnyError, usize),
    #[error("{1}")]
    IO(#[source] IoError, Cow<'static, str>),
}

/// Case mode to merging keys during (re)load.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum MergeCase {
    /// Auto detect merge case mode from appended parsers.
    #[default]
    Auto,
    /// Enforce case sensitive merge mode.
    Sensitive,
    /// Enforce case insensitive merge mode.
    Insensitive,
}

/// A data structure that has case-sensitive or case-insensitive keys.
pub trait Case {
    /// Return `true` if case sensitive, otherwise return `false`.
    #[inline]
    fn is_case_sensitive(&self) -> bool {
        true
    }
}

/// A data structure that can be parsed.
pub trait Parse: Case {
    /// Parse data to [`Value`] structure. The `value` parameter could hold merged results from previous parser(s)
    /// call(s). That merged `value` could be used to get some parameter(s) for current parse. For example, path to
    /// configuration file could be taken from previous command-line parser results (see `FileParser<L>::path_option` or
    /// `env::ParserBuilder::prefix_option`). If successful then data return as [`Value`] structure.
    ///
    /// # Errors
    ///
    /// If any errors will occur during parsing then error will be returned.
    fn parse(&mut self, value: &Value) -> AnyResult<Value>;
}

impl Case for AnyParser {
    #[inline]
    fn is_case_sensitive(&self) -> bool {
        self.as_ref().is_case_sensitive()
    }
}

impl Parse for AnyParser {
    #[inline]
    fn parse(&mut self, value: &Value) -> AnyResult<Value> {
        self.as_mut().parse(value)
    }
}

#[inline]
fn unicase(data: &str) -> String {
    data.to_lowercase()
}

#[inline]
fn normalize_case(data: &str, case_on: bool) -> CowString {
    if case_on {
        CowString::Borrowed(data)
    } else {
        CowString::Owned(unicase(data))
    }
}
