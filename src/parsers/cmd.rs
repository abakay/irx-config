//! This module provide command-line parser implementation based on [`clap`] crate.
//!
//! The value of each command-line option parsed will be typed according to `YAML` format.
//!
//! To enable that parser  one has to add the following to Cargo.toml:
//!
//! ```toml
//! [dependencies]
//! irx-config = { version = "1.0", features = ["cmd"] }
//! ```
//!
//! # Example
//!
//! ```no_run
//! use clap::App;
//! use irx_config::ConfigBuilder;
//! use irx_config::parsers::cmd::ParserBuilder;
//!
//! let yaml = clap::load_yaml!("cmd.yaml");
//! let matches = App::from_yaml(yaml).get_matches();
//!
//! let config = ConfigBuilder::default()
//!     .append_parser(
//!         ParserBuilder::default()
//!             .matches(matches)
//!             .try_arg_names_from_yaml(include_str!("cmd.yaml"))?
//!             .build()?,
//!     )
//!     .load()?;
//! ```

use crate::{AnyResult, Case, Parse, StdResult, Value, DEFAULT_KEYS_SEPARATOR};
use clap::ArgMatches;
use either::Either;
use serde_yaml::{Mapping, Value as YamlValue};

/// A result type for current module errors.
pub type Result<T> = StdResult<T, Error>;

/// Error generated during build parser process.
#[non_exhaustive]
#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Uninitialized field: {0:?}")]
    UninitializedField(&'static str),
    #[error("Yaml parser error")]
    ParseYaml(#[from] serde_yaml::Error),
    #[error(transparent)]
    Common(#[from] crate::Error),
}

/// The command-line parser implementation.
pub struct Parser {
    value: Value,
}

impl Case for Parser {
    #[inline]
    fn is_case_sensitive(&self) -> bool {
        self.value.is_case_sensitive()
    }
}

impl Parse for Parser {
    #[inline]
    fn parse(&mut self, _value: &Value) -> AnyResult<Value> {
        Ok(self.value.clone())
    }
}

/// Builder for [`Parser`].
pub struct ParserBuilder<'a> {
    matches: Option<ArgMatches<'a>>,
    arg_names: Vec<String>,
    keys_delimiter: String,
    case_sensitive: bool,
}

impl Default for ParserBuilder<'_> {
    fn default() -> Self {
        Self {
            matches: Default::default(),
            arg_names: Default::default(),
            keys_delimiter: DEFAULT_KEYS_SEPARATOR.to_string(),
            case_sensitive: true,
        }
    }
}

impl<'a> ParserBuilder<'a> {
    /// Set [`ArgMatches`] matches from [`clap`] crate.
    #[inline]
    pub fn matches(&mut self, matches: ArgMatches<'a>) -> &mut Self {
        self.matches = Some(matches);
        self
    }

    /// Set command-line arguments names from iterator.
    #[inline]
    pub fn arg_names<I, T>(&mut self, names: I) -> &mut Self
    where
        I: IntoIterator<Item = T>,
        T: Into<String>,
    {
        self.arg_names = names.into_iter().map(|n| n.into()).collect();
        self
    }

    /// Set command-lne arguments names from `YAML` file in [`clap`] format.
    pub fn try_arg_names_from_yaml<D>(&mut self, data: D) -> Result<&mut Self>
    where
        D: AsRef<str>,
    {
        fn inner(data: &str) -> Result<Vec<String>> {
            if let YamlValue::Mapping(ref m) = serde_yaml::from_str(data)? {
                let args = get_args_from_mapping(m);
                Ok(args.chain(get_args_from_subcommands(m)).cloned().collect())
            } else {
                Err(Error::Common(crate::Error::NotMap))
            }
        }

        self.arg_names = inner(data.as_ref())?;
        Ok(self)
    }

    /// Set key level delimiter.
    #[inline]
    pub fn keys_delimiter<S>(&mut self, delim: S) -> &mut Self
    where
        S: Into<String>,
    {
        self.keys_delimiter = delim.into();
        self
    }

    /// Set parser keys case sensitivity.
    #[inline]
    pub fn case_sensitive(&mut self, on: bool) -> &mut Self {
        self.case_sensitive = on;
        self
    }

    /// Build and return command-line parser [`Parser`].
    ///
    /// # Errors
    ///
    /// If any errors will occur during build then error will be returned.
    pub fn build(&self) -> Result<Parser> {
        let matches = self
            .matches
            .as_ref()
            .ok_or(Error::UninitializedField("matches"))?;
        let mut value = Value::with_case(self.case_sensitive);
        for arg in self.arg_names.iter() {
            if let Some(v) = matches.values_of_lossy(arg) {
                match v[..] {
                    [ref a] => {
                        value.set_by_key_path_with_delim(
                            arg,
                            &self.keys_delimiter,
                            serde_yaml::from_str::<YamlValue>(a)?,
                        )?;
                    }
                    _ => {
                        value.set_by_key_path_with_delim(
                            arg,
                            &self.keys_delimiter,
                            serde_yaml::from_str::<YamlValue>(&format!("[{}]", v.join(",")))?,
                        )?;
                    }
                }
            }
        }
        Ok(Parser { value })
    }
}

fn get_args_from_mapping(map: &Mapping) -> impl Iterator<Item = &String> {
    let args = YamlValue::String("args".to_string());
    if let Some(YamlValue::Sequence(args)) = map.get(&args) {
        Either::Left(args.iter().filter_map(|a| {
            if let YamlValue::Mapping(m) = a {
                m.iter().next().and_then(|(k, _)| match k {
                    YamlValue::String(s) => Some(s),
                    _ => None,
                })
            } else {
                None
            }
        }))
    } else {
        Either::Right(std::iter::empty())
    }
}

fn get_args_from_subcommands(map: &Mapping) -> impl Iterator<Item = &String> {
    let subcmds = YamlValue::String("subcommands".to_string());
    if let Some(YamlValue::Sequence(cmds)) = map.get(&subcmds) {
        Either::Left(
            cmds.iter()
                .filter_map(|c| {
                    if let YamlValue::Mapping(m) = c {
                        m.iter().next().and_then(|(_, v)| match v {
                            YamlValue::Mapping(m) => Some(get_args_from_mapping(m)),
                            _ => None,
                        })
                    } else {
                        None
                    }
                })
                .flatten(),
        )
    } else {
        Either::Right(std::iter::empty())
    }
}
