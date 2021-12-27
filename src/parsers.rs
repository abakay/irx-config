//! This module define base structures ([`FileParser`] and [`FileParserBuilder`]) which help to implement file based
//! parsers. All embedded file based parsers is using that base structures.

#[cfg(feature = "cmd")]
pub mod cmd;
#[cfg(feature = "env")]
pub mod env;
#[cfg(feature = "json")]
pub mod json;
#[cfg(test)]
mod tests;
#[cfg(feature = "toml-parser")]
pub mod toml;
#[cfg(feature = "yaml")]
pub mod yaml;

use crate::{AnyResult, Case, CowString, Parse, Value, DEFAULT_KEYS_SEPARATOR};
use derive_builder::Builder;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::PathBuf;

/// The trait to be used by [`FileParser`] to load data from file in specific format.
pub trait Load: Case {
    /// Load data from reader in specific format to [`Value`] structure.
    ///
    /// # Errors
    ///
    /// If any errors will occur during load then error will be returned.
    fn load(&mut self, reader: impl Read) -> AnyResult<Value>;
}

/// The base structure to implement file based parsers.
#[derive(Builder)]
#[builder(setter(into, strip_option))]
pub struct FileParser<L> {
    /// Set default path to the file to be parsed.
    default_path: PathBuf,
    /// Set path option name which could be used to get path value from previous parsing [`Value`] results.
    #[builder(default = "None")]
    path_option: Option<String>,
    /// Set delimiter used to separate keys levels in path value. Default is [`DEFAULT_KEYS_SEPARATOR`].
    #[builder(default = "DEFAULT_KEYS_SEPARATOR.to_string()")]
    keys_delimiter: String,
    /// Set the loader structure which implements [`Load`] trait.
    loader: L,
}

impl<L: Load> Case for FileParser<L> {
    #[inline]
    fn is_case_sensitive(&self) -> bool {
        self.loader.is_case_sensitive()
    }
}

impl<L: Load> Parse for FileParser<L> {
    fn parse(&mut self, value: &Value) -> AnyResult<Value> {
        let path = if let Some(ref p) = self.path_option {
            value.get_by_key_path_with_delim(p, &self.keys_delimiter)?
        } else {
            None
        }
        .map(CowString::Owned)
        .unwrap_or_else(|| self.default_path.to_string_lossy());

        let reader = BufReader::new(File::open(path.as_ref())?);
        self.loader.load(reader)
    }
}
