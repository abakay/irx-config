//! This module define base structures ([`FileParser`] and [`FileParserBuilder`]) which help to implement file based
//! parsers. All embedded file based parsers is using that base structures.

#[cfg(feature = "cmd")]
pub mod cmd;
#[cfg(feature = "env")]
pub mod env;
#[cfg(feature = "json")]
pub mod json;
#[cfg(feature = "json5-parser")]
pub mod json5;
#[cfg(test)]
mod tests;
#[cfg(feature = "toml-parser")]
pub mod toml;
#[cfg(feature = "yaml")]
pub mod yaml;

use crate::{AnyResult, Case, Parse, Value, DEFAULT_KEYS_SEPARATOR};
use derive_builder::Builder;
use std::{
    borrow::Cow,
    fs::File,
    io::{BufReader, Error as IoError, Read},
    path::{Path, PathBuf},
    result::Result as StdResult,
};

/// A result type for file-based parsers errors.
pub type Result<T> = StdResult<T, Error>;

type CowPath<'a> = Cow<'a, Path>;

/// All errors for file-based parsers.
#[non_exhaustive]
#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Failed to get file path by option: '{1}'")]
    PathOption(#[source] crate::Error, String),
    #[error("Failed to open file: '{1}'")]
    Open(#[source] IoError, PathBuf),
    #[error("Failed to get meta data for file: '{1}'")]
    Meta(#[source] IoError, PathBuf),
    #[error("Is not a file: '{0}'")]
    NotAFile(PathBuf),
}

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
pub struct FileParser<L: Load + Default> {
    /// Set default path to the file to be parsed.
    default_path: PathBuf,
    /// Set path option name which could be used to get path value from previous parsing [`Value`] results.
    #[builder(default = "None")]
    path_option: Option<String>,
    /// Set delimiter used to separate keys levels in path value. Default is [`DEFAULT_KEYS_SEPARATOR`].
    #[builder(default = "DEFAULT_KEYS_SEPARATOR.to_string()")]
    keys_delimiter: String,
    /// If file does not exists do not try to load it. The default [`Value`] will be returned. Default is `false`.
    #[builder(default = "false")]
    ignore_missing_file: bool,
    /// Set the loader structure which implements [`Load`] trait.
    #[builder(default)]
    loader: L,
}

impl<L: Load + Default> Case for FileParser<L> {
    #[inline]
    fn is_case_sensitive(&self) -> bool {
        self.loader.is_case_sensitive()
    }
}

impl<L: Load + Default> Parse for FileParser<L> {
    fn parse(&mut self, value: &Value) -> AnyResult<Value> {
        let path = get_path(
            value,
            &self.path_option,
            &self.default_path,
            &self.keys_delimiter,
        )?;

        let file = match try_open_file(path.as_ref()) {
            Ok(f) => f,
            Err(_) if self.ignore_missing_file => return Ok(Value::default()),
            Err(e) => return Err(e.into()),
        };

        self.loader.load(BufReader::new(file))
    }
}

fn get_path<'a>(
    value: &Value,
    path_option: &Option<String>,
    default: &'a Path,
    delim: &str,
) -> Result<CowPath<'a>> {
    let default = default.into();
    let Some(option) = path_option else {
        return Ok(default);
    };

    let path: Option<String> = value
        .get_by_key_path_with_delim(option, delim)
        .map_err(|e| Error::PathOption(e, option.into()))?;
    Ok(path.map_or(default, |p| PathBuf::from(p).into()))
}

fn try_open_file(path: &Path) -> Result<File> {
    let file = File::open(path).map_err(|e| Error::Open(e, path.into()))?;
    if file
        .metadata()
        .map_err(|e| Error::Meta(e, path.into()))?
        .is_file()
    {
        return Ok(file);
    }

    Err(Error::NotAFile(path.into()))
}
