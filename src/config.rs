//! This module define main configuration structures: [`Config`] and [`ConfigBuilder`].

use crate::{AnyParser, Error, MergeCase, Parse, Result, Value, DEFAULT_KEYS_SEPARATOR};
use serde::de::DeserializeOwned;
use std::cmp::Ordering;
use std::fmt::{Debug, Display, Formatter, Result as FmtResult};

cfg_if::cfg_if! {
    if #[cfg(feature = "blake2b")] {
        use blake2b_simd as HashModule;
        use HashModule::blake2b as hash_func;
        const HASH_NAME: &str = "BLAKE2b";
    } else if #[cfg(feature = "blake3")] {
        use blake3 as HashModule;
        use HashModule::hash as hash_func;
        const HASH_NAME: &str = "BLAKE3";
    }
}

#[derive(PartialEq)]
struct Hash(HashModule::Hash);

impl Hash {
    #[inline]
    fn as_bytes(&self) -> &[u8] {
        self.0.as_bytes()
    }
}

impl From<&[u8]> for Hash {
    #[inline]
    fn from(value: &[u8]) -> Self {
        Self(hash_func(value))
    }
}

impl Debug for Hash {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        Debug::fmt(&self.0, f)
    }
}

impl Display for Hash {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.write_str(&self.0.to_hex())
    }
}

/// Container for all parser sources which will (re)load data from a parsers in order in which they was added
/// to [`ConfigBuilder`]. It will provide access to merged set of (re)loaded configuration parameters.
pub struct Config {
    parsers: Vec<AnyParser>,
    value: Value,
    case_on: bool,
    hash: Hash,
    sealed_suffix: String,
    keys_delimiter: String,
}

impl Config {
    /// Reload and re-merge all configuration data from parsers.
    ///
    /// # Errors
    ///
    /// If any errors will occur during parsing/merging then error will be returned.
    pub fn reload(&mut self) -> Result<&mut Self> {
        let mut value = Value::default();
        for (idx, parser) in self.parsers.iter_mut().enumerate() {
            value = parser
                .parse(&value)
                .map_err(|e| Error::ParseValue(e, idx + 1))?
                .merge_with_case(&value, self.case_on);
        }

        value.seal(&self.sealed_suffix);
        self.hash = Hash::from(value.as_bytes().as_ref());
        self.value = value;
        Ok(self)
    }

    /// Name of the hash used for loaded configuration data.
    #[inline]
    pub fn hash_name() -> &'static str {
        HASH_NAME
    }

    /// Calculate hash for currently loaded configuration data.
    #[inline]
    pub fn hash(&self) -> String {
        [HASH_NAME, ": ", &self.hash.to_string()].concat()
    }

    /// Returns configuration data value to corresponding key/nested keys.
    ///
    /// # Example
    ///
    /// ```
    /// let name: Option<u32> = conf.get_by_keys(["logger", "name"])?;
    /// ```
    ///
    /// # Errors
    ///
    /// If keys is empty, the error will be returned.
    #[inline]
    pub fn get_by_keys<I, K, T>(&self, keys: I) -> Result<Option<T>>
    where
        I: IntoIterator<Item = K>,
        K: AsRef<str>,
        T: DeserializeOwned,
    {
        self.value.get_by_keys(keys)
    }

    /// Returns configuration data value to corresponding key path with keys delimiter. Default delimiter is
    /// [`DEFAULT_KEYS_SEPARATOR`].
    ///
    /// # Example
    ///
    /// ```
    /// use serde::Deserialize;
    ///
    /// #[derive(Deserialize)]
    /// struct Person {
    ///     first_name: String,
    ///     last_name: String,
    ///     age: u8,
    /// }
    ///
    /// let person: Option<Person> = conf.get_by_key_path("contact:info")?;
    /// ```
    ///
    /// # Errors
    ///
    /// If keys path or keys delimiter is empty, the corresponding error will be returned.
    #[inline]
    pub fn get_by_key_path<T, P>(&self, path: P) -> Result<Option<T>>
    where
        T: DeserializeOwned,
        P: AsRef<str>,
    {
        self.value
            .get_by_key_path_with_delim(path, &self.keys_delimiter)
    }

    /// Returns configuration data value to corresponding key path with delimiter.
    ///
    /// # Example
    ///
    /// ```
    /// let name: Option<u32> = conf.get_by_key_path_with_delim("logger:name", ":")?;
    /// ```
    ///
    /// # Errors
    ///
    /// If keys path or delimiter is empty, the corresponding error will be returned.
    #[inline]
    pub fn get_by_key_path_with_delim<T, P, D>(&self, path: P, delim: D) -> Result<Option<T>>
    where
        T: DeserializeOwned,
        P: AsRef<str>,
        D: AsRef<str>,
    {
        self.value.get_by_key_path_with_delim(path, delim)
    }

    /// Deserialize configuration to destination struct/value.
    ///
    /// # Example
    ///
    /// ```
    /// use serde::Deserialize;
    ///
    /// #[derive(Deserialize)]
    /// struct Person {
    ///     first_name: String,
    ///     last_name: String,
    ///     age: u8,
    /// }
    ///
    /// let person: Person = conf.get()?;
    /// ```
    ///
    /// # Errors
    ///
    /// In case of any de-serialization problems the corresponding error will be returned.
    #[inline]
    pub fn get<T: DeserializeOwned>(&self) -> Result<T> {
        self.value.get()
    }

    /// Get reference to internal [`Value`] structure.
    #[inline]
    pub fn get_value(&self) -> &Value {
        &self.value
    }
}

impl Debug for Config {
    #[inline]
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        f.write_fmt(format_args!(
            "Config {{ parsers: size({}), value: {:?}, case_on: {:?}, hash: {:?}, sealed_suffix: {:?}, keys_delimiter: {:?} }}",
            self.parsers.len(),
            self.value,
            self.case_on,
            self.hash,
            self.sealed_suffix,
            self.keys_delimiter,
        ))
    }
}

impl Display for Config {
    #[inline]
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        f.write_fmt(format_args!("Config: {}\n{}", self.hash(), self.value))
    }
}

impl PartialEq for Config {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.hash == other.hash
    }
}

impl Eq for Config {}

impl PartialOrd for Config {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Config {
    #[inline]
    fn cmp(&self, other: &Self) -> Ordering {
        self.hash.as_bytes().cmp(other.hash.as_bytes())
    }
}

/// The builder for [`Config`] structure.
pub struct ConfigBuilder {
    parsers: Vec<AnyParser>,
    sealed_suffix: String,
    keys_delimiter: String,
    auto_case_on: bool,
    merge_case: MergeCase,
}

impl ConfigBuilder {
    /// Append a parser to [`Config`]. First appended parser will have highest priority during (re)load merge, the last
    /// one will have lowest priority.
    ///
    /// # Example
    ///
    /// ```
    /// use irx_config::parsers::{env, json};
    /// use irx_config::ConfigBuilder;
    ///
    /// let config = ConfigBuilder::default()
    ///     .append_parser(
    ///         json::ParserBuilder::default()
    ///             .default_path("config.json")
    ///             .build()?,
    ///     )
    ///     .append_parser(
    ///         env::ParserBuilder::default()
    ///             .default_prefix("APP_")
    ///             .build()?,
    ///     )
    ///     .load()?;
    /// ```
    #[inline]
    pub fn append_parser<P>(mut self, parser: P) -> Self
    where
        P: Parse + 'static,
    {
        self.auto_case_on = self.auto_case_on && parser.is_case_sensitive();
        self.parsers.push(Box::new(parser));
        self
    }

    /// Set suffix for keys to mark them as a secret value which will be obfuscated during display/debugging output.
    /// If not set then all values will be displayed as is.
    ///
    ///
    /// # Example
    ///
    /// ```
    /// use irx_config::parsers::env;
    /// use irx_config::ConfigBuilder;
    ///
    /// let config = ConfigBuilder::default()
    ///     .append_parser(
    ///         env::ParserBuilder::default()
    ///             .default_prefix("APP_")
    ///             .build()?,
    ///     )
    ///     .sealed_suffix("_sealed_")
    ///     .load()?;
    /// ```
    #[inline]
    pub fn sealed_suffix<S>(mut self, suffix: S) -> Self
    where
        S: Into<String>,
    {
        self.sealed_suffix = suffix.into();
        self
    }

    /// Set default key level delimiter. Default is [`DEFAULT_KEYS_SEPARATOR`].
    ///
    /// # Example
    ///
    /// ```
    /// use clap::app_from_crate;
    /// use irx_config::parsers::cmd;
    /// use irx_config::ConfigBuilder;
    ///
    /// let app = app_from_crate!();
    ///
    /// let config = ConfigBuilder::default()
    ///     .append_parser(cmd::ParserBuilder::new(app).build()?)
    ///     .keys_delimiter("/")
    ///     .load()?;
    /// ```
    #[inline]
    pub fn keys_delimiter<D>(mut self, delim: D) -> Self
    where
        D: Into<String>,
    {
        self.keys_delimiter = delim.into();
        self
    }

    /// Set merge case mode for a keys (see [`MergeCase`]). Default is [`MergeCase::Auto`].
    #[inline]
    pub fn merge_case(mut self, case: MergeCase) -> Self {
        self.merge_case = case;
        self
    }

    /// Load all data from all previously appended parsers, merge data according to appended order and return [`Config`].
    ///
    /// # Errors
    ///
    /// If any errors will occur during parsing/merging then error will be returned.
    pub fn load(self) -> Result<Config> {
        let value = Value::default();
        let hash = Hash::from(value.as_bytes().as_ref());
        let case_on = if MergeCase::Auto == self.merge_case {
            self.auto_case_on
        } else {
            MergeCase::Sensitive == self.merge_case
        };

        let mut config = Config {
            parsers: self.parsers,
            value,
            case_on,
            hash,
            sealed_suffix: self.sealed_suffix,
            keys_delimiter: self.keys_delimiter,
        };
        config.reload()?;
        Ok(config)
    }

    /// Load data from one parser and return [`Config`].
    ///
    /// # Errors
    ///
    /// If any errors will occur during parsing/merging then error will be returned.
    #[inline]
    pub fn load_one<P>(parser: P) -> Result<Config>
    where
        P: Parse + 'static,
    {
        ConfigBuilder::default().append_parser(parser).load()
    }

    /// Load all data from parsers' iterator, merge data according to iterator order and return [`Config`].
    ///
    /// # Errors
    ///
    /// If any errors will occur during parsing/merging then error will be returned.
    #[inline]
    pub fn load_from<I, P>(parsers: I) -> Result<Config>
    where
        I: IntoIterator<Item = P>,
        P: Parse + 'static,
    {
        parsers
            .into_iter()
            .fold(ConfigBuilder::default(), |s, p| s.append_parser(p))
            .load()
    }
}

impl Default for ConfigBuilder {
    fn default() -> Self {
        Self {
            parsers: Default::default(),
            sealed_suffix: Default::default(),
            keys_delimiter: DEFAULT_KEYS_SEPARATOR.to_string(),
            auto_case_on: true,
            merge_case: Default::default(),
        }
    }
}
