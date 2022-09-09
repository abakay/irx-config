//! This module provide command-line parser implementation based on [`clap`](https://docs.rs/clap/latest/clap/) crate.
//!
//! The value of each command-line option parsed will be typed according to `YAML` format.
//!
//! To enable that parser one has to add the following to Cargo.toml:
//!
//! ```toml
//! [dependencies]
//! irx-config = { version = "2.2", features = ["cmd"] }
//! ```
//!
//! # Examples
//!
//! The names of arguments could contains keys delimiter (see [`DEFAULT_KEYS_SEPARATOR`] or/and
//! [`ParserBuilder::keys_delimiter`] method). In such case argument's names will be splitted to nested keys and will
//! be merged accordingly with other parsers results. Such nested keys structure(s) will represent
//! sub dictionaries/sections.
//!
//! ```no_run
//! use clap::{command, Arg};
//! use irx_config::ConfigBuilder;
//! use irx_config::parsers::cmd::ParserBuilder;
//!
//! let command = command!()
//!             .arg(Arg::new("settings:host").short('H').long("host").takes_value(true));
//!
//! let config = ConfigBuilder::default()
//!     .append_parser(ParserBuilder::new(command).build()?)
//!     .load()?;
//! ```
//!
//! If `global key names` feature is off (see [`ParserBuilder::global_key_names`] method) and executable has
//! subcommand(s) then arguments names for each subcommand will be prefixed with given subcommand's name and keys
//! delimiter. The `global key names` feature is on by default.
//!
//! ```no_run
//! use clap::{command, Command, Arg};
//! use irx_config::ConfigBuilder;
//! use irx_config::parsers::cmd::ParserBuilder;
//!
//! let command = command!()
//!             .subcommand(
//!                 Command::new("connect")
//!                     .arg(Arg::new("host").short('H').long("host").takes_value(true))
//!             );
//!
//! let config = ConfigBuilder::default()
//!     .append_parser(ParserBuilder::new(command).global_key_names(false).build()?)
//!     .load()?;
//! ```

use crate::{AnyResult, Case, Parse, StdResult, Value, DEFAULT_KEYS_SEPARATOR};
use clap::{Arg, ArgMatches, Command};
use serde_yaml::Value as YamlValue;
use std::{borrow::Cow, env, ffi::OsString};

/// A result type for current module errors.
pub type Result<T> = StdResult<T, Error>;

/// The default maximum depth to get (sub)commands arguments names.
pub const DEFAULT_MAX_DEPTH: u8 = 16;

/// Error generated during build parser process.
#[non_exhaustive]
#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Yaml parser error")]
    ParseYaml(#[from] serde_yaml::Error),
    #[error("Clap error")]
    Clap(#[from] clap::Error),
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
    command: Command<'a>,
    args: Option<Vec<OsString>>,
    global_key_names: bool,
    max_depth: u8,
    keys_delimiter: String,
    case_sensitive: bool,
    exit_on_error: bool,
    single_flags_as_bool: bool,
}

impl<'a> ParserBuilder<'a> {
    /// Create [`ParserBuilder`] from `clap::Command` instance.
    #[inline]
    pub fn new(command: Command<'a>) -> Self {
        Self {
            command,
            args: Default::default(),
            global_key_names: true,
            max_depth: DEFAULT_MAX_DEPTH,
            keys_delimiter: DEFAULT_KEYS_SEPARATOR.to_string(),
            case_sensitive: true,
            exit_on_error: false,
            single_flags_as_bool: false,
        }
    }

    /// Set arguments to be parsed, otherwise program command-line arguments will be used.
    #[inline]
    pub fn args<I, T>(&mut self, args: I) -> &mut Self
    where
        I: IntoIterator<Item = T>,
        T: Into<OsString>,
    {
        self.args = Some(args.into_iter().map(|a| a.into()).collect());
        self
    }

    /// Use global key names for arguments if `true`, otherwise (sub)command name(s) will be used to prefix arguments
    /// keys names with keys names delimiter. Default is `true`.
    #[inline]
    pub fn global_key_names(&mut self, on: bool) -> &mut Self {
        self.global_key_names = on;
        self
    }

    /// Max depth of subcommands arguments to parse. Default is [`DEFAULT_MAX_DEPTH`].
    #[inline]
    pub fn max_depth(&mut self, depth: u8) -> &mut Self {
        self.max_depth = depth;
        self
    }

    /// Set key level delimiter. Default is [`DEFAULT_KEYS_SEPARATOR`].
    #[inline]
    pub fn keys_delimiter<S>(&mut self, delim: S) -> &mut Self
    where
        S: Into<String>,
    {
        self.keys_delimiter = delim.into();
        self
    }

    /// Set parser keys case sensitivity. Default is `true`.
    #[inline]
    pub fn case_sensitive(&mut self, on: bool) -> &mut Self {
        self.case_sensitive = on;
        self
    }

    /// If set to `true` then exit from program on `clap::Error` during building stage. Default is `false`.
    #[inline]
    pub fn exit_on_error(&mut self, on: bool) -> &mut Self {
        self.exit_on_error = on;
        self
    }

    /// By default all command-line parameters without values will have they values set to number of they occurrences.
    /// If this setting is set to `true` and a parameter **not allowed to have multiple occurrences** by `clap` API
    /// then parameter's value will have boolean `true` as a value.
    ///
    /// **NOTE**: That default values for that setting is `false` now, but it probably will be changed to `true`
    /// in next major release version to provide more ergonomic API.
    #[inline]
    pub fn single_flags_as_bool(&mut self, on: bool) -> &mut Self {
        self.single_flags_as_bool = on;
        self
    }

    /// Build and return command-line parser [`Parser`].
    ///
    /// # Errors
    ///
    /// If any errors will occur during build then error will be returned. If `exit_on_error` was set to `true` then
    /// program will exit if `clap::Error` will occur. Otherwise [`Result`] with proper error will be returned.
    pub fn build(&mut self) -> Result<Parser> {
        let result = self.get_matches();
        let matches = if self.exit_on_error {
            result.unwrap_or_else(|e| e.exit())
        } else {
            result?
        };

        let value = self.get_app_arguments(
            Value::with_case(self.case_sensitive),
            &self.command,
            &matches,
            "",
            self.max_depth,
        )?;

        Ok(Parser { value })
    }

    fn get_matches(&mut self) -> clap::Result<ArgMatches> {
        if let Some(ref args) = self.args {
            self.command.try_get_matches_from_mut(args)
        } else {
            self.command.try_get_matches_from_mut(&mut env::args_os())
        }
    }

    fn get_app_arguments(
        &self,
        mut value: Value,
        command: &Command,
        matches: &ArgMatches,
        app_name: &str,
        depth: u8,
    ) -> Result<Value> {
        let prefix = if self.global_key_names || app_name.is_empty() {
            Default::default()
        } else {
            [app_name, &self.keys_delimiter].concat()
        };

        for arg in command.get_arguments() {
            value = set_value(
                value,
                matches,
                &[&prefix, arg.get_id()].concat(),
                &self.keys_delimiter,
                arg,
                self.single_flags_as_bool,
            )?;
        }

        if depth == 0 {
            return Ok(value);
        }

        for a in command.get_subcommands() {
            if let Some(m) = matches.subcommand_matches(a.get_name()) {
                value = self.get_app_arguments(
                    value,
                    a,
                    m,
                    &[&prefix, a.get_name()].concat(),
                    depth - 1,
                )?;
            }
        }

        Ok(value)
    }
}

fn get_values(matches: &ArgMatches, arg: &Arg) -> Option<Vec<String>> {
    let id = arg.get_id();
    if arg.is_allow_invalid_utf8_set() {
        return matches.values_of_lossy(id);
    }

    matches
        .values_of(id)
        .map(|v| v.map(|i| i.to_string()).collect())
}

fn set_value(
    mut value: Value,
    matches: &ArgMatches,
    path: &str,
    delim: &str,
    arg: &Arg,
    single_flags_as_bool: bool,
) -> Result<Value> {
    if let Some(v) = get_values(matches, arg) {
        let is_list = arg.is_multiple_values_set()
            || arg.is_multiple_occurrences_set()
            || arg.is_use_value_delimiter_set();
        let v: Vec<_> = v
            .into_iter()
            .map(|i| if i.is_empty() { "''".to_string() } else { i })
            .collect();

        let v = match v[..] {
            [ref a] if !is_list => Cow::Borrowed(a),
            [] if !arg.is_takes_value_set() => {
                if single_flags_as_bool && !arg.is_multiple_occurrences_set() {
                    Cow::Owned("true".to_string())
                } else {
                    Cow::Owned(matches.occurrences_of(arg.get_id()).to_string())
                }
            }
            _ => Cow::Owned(["[", &v.join(","), "]"].concat()),
        };
        value.set_by_key_path_with_delim(path, delim, serde_yaml::from_str::<YamlValue>(&v)?)?;
    }
    Ok(value)
}
