#![doc = include_str!("../README.md")]

pub mod config;
#[cfg(feature = "parsers")]
pub mod parsers;
#[cfg(test)]
mod tests;
pub mod value;

pub use crate::config::{Config, ConfigBuilder};
pub use crate::value::json;
use crate::value::SerdeError;
pub use crate::value::Value;
use std::borrow::Cow;
use std::error::Error as StdError;
use std::fmt::Debug;
use std::result::Result as StdResult;

/// A result type with internal error.
pub type Result<T> = StdResult<T, Error>;

/// A result type with any error.
pub type AnyResult<T> = StdResult<T, Box<dyn StdError>>;

type CowString<'a> = Cow<'a, str>;
type AnyParser = Box<dyn Parse>;

/// Default key level separator.
pub const DEFAULT_KEYS_SEPARATOR: &str = ":";

/// Error generated during any crate operations.
#[non_exhaustive]
#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Empty key path separator")]
    EmptySeparator,
    #[error("Mapping object expected")]
    NotMap,
    #[error("Failed to serialize/deserialize")]
    SerdeError(#[from] SerdeError),
    #[error("Failed to parse value")]
    ParseValue(#[from] Box<dyn StdError>),
    #[error(transparent)]
    IO(#[from] std::io::Error),
}

/// Case mode to merging keys during (re)load.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MergeCase {
    /// Auto detect merge case mode from appended parsers.
    Auto,
    /// Enforce case sensitive merge mode.
    Sensitive,
    /// Enforce case insensitive merge mode.
    Insensitive,
}

impl Default for MergeCase {
    fn default() -> Self {
        Self::Auto
    }
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
