use crate::{json, AnyResult, ConfigBuilder, Value};
use std::fs;

#[macro_export]
macro_rules! resource_dir {
    () => {
        concat!(env!("CARGO_MANIFEST_DIR"), "/tests/resources")
    };
}

#[macro_export]
macro_rules! resource_path {
    ($name:expr) => {
        concat!(resource_dir!(), "/", $name)
    };
}

#[cfg(feature = "env")]
mod env_test {
    use super::*;
    use crate::parsers::env::ParserBuilder;
    use std::env;

    fn setup() {
        env::set_var("HOME", "/home/joe");
        env::set_var("APP_ID", "42");
        env::set_var("APP_NODE1__ID", "1");
        env::set_var("APP_NODE1__NAMES", "[master, '1']");
    }

    #[test]
    fn parser() -> AnyResult<()> {
        setup();

        let expected = Value::try_from(json!(
        {
            "id": 42,
            "node1": {
                "id": 1,
                "names": ["master", "1"]
            }
        }))?;

        let parsers = [ParserBuilder::default()
            .default_prefix("APP_")
            .prefix_option("prefix")
            .build()?];
        let conf = ConfigBuilder::load_from(parsers)?;
        assert_eq!(expected, conf.get::<Value>()?);
        Ok(())
    }

    #[test]
    fn parser_case_sensitive() -> AnyResult<()> {
        setup();

        let expected = Value::try_from(json!(
        {
            "ID": 42,
            "NODE1": {
                "ID": 1,
                "NAMES": ["master", "1"]
            }
        }))?;

        let parser = ParserBuilder::default()
            .default_prefix("APP_")
            .prefix_option("prefix")
            .case_sensitive(true)
            .build()?;
        let conf = ConfigBuilder::load_one(parser)?;
        assert_eq!(expected, conf.get::<Value>()?);
        Ok(())
    }
}

#[cfg(feature = "json")]
mod json_test {
    use super::*;
    use crate::parsers::json::ParserBuilder;

    #[test]
    fn parser() -> AnyResult<()> {
        let path = resource_path!("config.json");
        let expected = fs::read_to_string(&path)?;
        let expected: Value = serde_json::from_str(&expected)?;

        let conf = ConfigBuilder::default()
            .append_parser(
                ParserBuilder::default()
                    .default_path(path)
                    .path_option("config")
                    .build()?,
            )
            .load()?;
        assert_eq!(expected, conf.get::<Value>()?);
        Ok(())
    }
}

#[cfg(feature = "yaml")]
mod yaml_test {
    use super::*;
    use crate::parsers::yaml::ParserBuilder;
    use std::path::Path;

    #[test]
    fn parser() -> AnyResult<()> {
        let path = Path::new(resource_dir!()).join("config.yaml");
        let expected = fs::read_to_string(&path)?;
        let expected: Value = serde_yaml::from_str(&expected)?;

        let conf =
            ConfigBuilder::load_from(vec![ParserBuilder::default().default_path(path).build()?])?;
        assert_eq!(expected, conf.get::<Value>()?);
        Ok(())
    }
}

#[cfg(feature = "toml-parser")]
mod toml_test {
    use super::*;
    use crate::parsers::toml::ParserBuilder;

    #[test]
    fn parser() -> AnyResult<()> {
        let path = resource_path!("config.toml");
        let expected = Value::try_from(json!({
            "id": 42,
            "node1": {
                "id": 1,
                "names": ["master", "1"],
                "timestamp": "2021-09-12T20:11:45Z",
                "date": "2021-09-12",
                "time": "20:11:45",
                "factor": 1.3
            },
            "persons": [
                {
                    "name": "John Doe",
                    "age": 42
                },
                {
                    "name": "Jane Doe",
                    "age": 24
                }
            ]
        }))?;
        println!("expected: {:?}", expected);

        let conf = ConfigBuilder::load_one(
            ParserBuilder::default()
                .default_path(path)
                .path_option("config")
                .build()?,
        )?;
        let calculated: Value = conf.get()?;
        println!("calculated: {:?}", calculated);

        assert_eq!(expected, calculated);
        Ok(())
    }
}

#[cfg(feature = "cmd")]
mod test_cmd {
    use super::*;
    use crate::parsers::cmd::ParserBuilder;
    use clap::{Arg, Command};

    fn create_app_with_subcmds<'a>() -> Command<'a> {
        let user_args = [
            Arg::new("name:first").short('n').takes_value(true),
            Arg::new("enabled").short('e').global(true),
        ];

        let alias_args = [
            Arg::new("name").short('a').takes_value(true).required(true),
            Arg::new("enabled").short('e').global(true),
        ];

        Command::new("test")
            .arg(
                Arg::new("config")
                    .short('c')
                    .long("config")
                    .takes_value(true)
                    .required(true),
            )
            .subcommand(
                Command::new("user")
                    .subcommand(Command::new("add").args(&user_args))
                    .subcommand(Command::new("del").args(&user_args)),
            )
            .subcommand(
                Command::new("alias")
                    .subcommand(Command::new("add").args(&alias_args))
                    .subcommand(Command::new("del").args(&alias_args)),
            )
    }

    #[test]
    fn parser() -> AnyResult<()> {
        let expected = Value::try_from(json!({
            "name": "Joe",
            "timeout": ["", 1000, "2000", false],
            "verbose": 3,
            "tag": ""
        }))?;
        println!("expected: {:?}", expected);

        let command = Command::new("test").args([
            Arg::new("name")
                .short('n')
                .long("name")
                .takes_value(true)
                .required(true),
            Arg::new("timeout")
                .multiple_occurrences(true)
                .short('t')
                .long("timeout")
                .takes_value(true),
            Arg::new("tag").short('T').takes_value(true),
            Arg::new("verbose").multiple_occurrences(true).short('v'),
        ]);
        let args = [
            "test",
            "-n",
            "Joe",
            "-t",
            "''",
            "-t",
            "1000",
            "-v",
            "--timeout",
            "'2000'",
            "-t",
            "false",
            "-vv",
            "-T",
            "''",
        ];
        let conf = ConfigBuilder::load_one(ParserBuilder::new(command).args(args).build()?)?;
        let calculated = conf.get_value();
        println!("calculated: {:?}", calculated);
        assert_eq!(expected, *calculated);
        Ok(())
    }

    fn user_add(expected: Value, global_on: bool) -> AnyResult<()> {
        println!("expected: {:?}", expected);

        let app = create_app_with_subcmds();
        let args = [
            "test",
            "--config",
            "config.toml",
            "user",
            "add",
            "-n",
            "John",
            "-e",
        ];
        let conf = ConfigBuilder::load_one(
            ParserBuilder::new(app)
                .args(args)
                .global_key_names(global_on)
                .build()?,
        )?;
        let calculated = conf.get_value();
        println!("calculated: {:?}", calculated);
        assert_eq!(expected, *calculated);
        Ok(())
    }

    #[test]
    fn user_add_without_global_names() -> AnyResult<()> {
        let expected = Value::try_from(json!({
            "config": "config.toml",
            "user": {
                "add": {
                    "name": {
                        "first": "John"
                    },
                    "enabled": 1
                }
            }
        }))?;

        user_add(expected, false)
    }

    #[test]
    fn user_add_with_global_names() -> AnyResult<()> {
        let expected = Value::try_from(json!({
            "config": "config.toml",
            "name": {
                "first": "John"
            },
            "enabled": 1
        }))?;

        user_add(expected, true)
    }
}
