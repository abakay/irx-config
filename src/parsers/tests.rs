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
        let expected = fs::read_to_string(path)?;
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

    #[test]
    fn parser_ignore_missing_file() -> AnyResult<()> {
        let path = resource_path!("missing.json");
        let conf = ConfigBuilder::default()
            .append_parser(
                ParserBuilder::default()
                    .default_path(path)
                    .ignore_missing_file(true)
                    .build()?,
            )
            .load()?;
        assert_eq!(Value::default(), conf.get::<Value>()?);
        Ok(())
    }

    #[test]
    #[should_panic(expected = "kind: NotFound")]
    fn parse_file_not_found() {
        let path = resource_path!("not-found.json");
        ConfigBuilder::default()
            .append_parser(
                ParserBuilder::default()
                    .default_path(path)
                    .path_option("config")
                    .build()
                    .unwrap(),
            )
            .load()
            .unwrap();
    }

    #[test]
    #[should_panic(expected = "Is not a file")]
    fn parse_not_a_file() {
        ConfigBuilder::default()
            .append_parser(
                ParserBuilder::default()
                    .default_path(".")
                    .path_option("config")
                    .build()
                    .unwrap(),
            )
            .load()
            .unwrap();
    }
}

#[cfg(feature = "json5-parser")]
mod json5_test {
    use super::*;
    use crate::parsers::json5::ParserBuilder;

    #[test]
    fn parser() -> AnyResult<()> {
        let path = resource_path!("config.json5");
        let expected_path = resource_path!("config.json");
        let expected = fs::read_to_string(expected_path)?;
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
    use clap::{value_parser, Arg, ArgAction, Command};

    fn create_app_with_subcmds() -> Command {
        let user_args = [
            Arg::new("name:first").short('n'),
            Arg::new("enabled")
                .short('e')
                .action(ArgAction::SetTrue)
                .global(true),
        ];

        let alias_args = [
            Arg::new("name").short('a').required(true),
            Arg::new("enabled")
                .short('e')
                .action(ArgAction::SetTrue)
                .global(true),
        ];

        Command::new("test")
            .arg(Arg::new("config").short('c').long("config").required(true))
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
            "debug": 1,
            "tag": ""
        }))?;
        println!("expected: {:?}", expected);

        let command = Command::new("test").args([
            Arg::new("name").short('n').long("name").required(true),
            Arg::new("timeout")
                .action(ArgAction::Append)
                .short('t')
                .long("timeout"),
            Arg::new("tag").short('T'),
            Arg::new("verbose").action(ArgAction::Count).short('v'),
            Arg::new("debug").short('d').action(ArgAction::Count),
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
            "-d",
        ];
        let conf = ConfigBuilder::load_one(
            ParserBuilder::new(command)
                .args(args)
                .use_arg_types(false)
                .build()?,
        )?;
        let calculated = conf.get_value();
        println!("calculated: {:?}", calculated);
        assert_eq!(expected, *calculated);
        Ok(())
    }

    #[test]
    fn use_value_delimiter() -> AnyResult<()> {
        let expected = Value::try_from(json!({
            "users": ["joe", "john"],
        }))?;
        println!("expected: {:?}", expected);

        let command =
            Command::new("test").args([Arg::new("users").value_delimiter(',').short('u')]);
        let args = ["test", "-u", "joe,john"];
        let conf = ConfigBuilder::load_one(ParserBuilder::new(command).args(args).build()?)?;
        let calculated = conf.get_value();
        println!("calculated: {:?}", calculated);
        assert_eq!(expected, *calculated);
        Ok(())
    }

    #[test]
    fn use_value_delimiter_single() -> AnyResult<()> {
        let expected = Value::try_from(json!({
            "users": ["joe"],
        }))?;
        println!("expected: {:?}", expected);

        let command =
            Command::new("test").args([Arg::new("users").value_delimiter(',').short('u')]);
        let args = ["test", "-u", "joe"];
        let conf = ConfigBuilder::load_one(ParserBuilder::new(command).args(args).build()?)?;
        let calculated = conf.get_value();
        println!("calculated: {:?}", calculated);
        assert_eq!(expected, *calculated);
        Ok(())
    }

    #[test]
    fn not_use_value_delimiter() -> AnyResult<()> {
        let expected = Value::try_from(json!({
            "users": "joe,john",
        }))?;
        println!("expected: {:?}", expected);

        let command =
            Command::new("test").args([Arg::new("users").value_delimiter(None).short('u')]);
        let args = ["test", "-u", "joe,john"];
        let conf = ConfigBuilder::load_one(ParserBuilder::new(command).args(args).build()?)?;
        let calculated = conf.get_value();
        println!("calculated: {:?}", calculated);
        assert_eq!(expected, *calculated);
        Ok(())
    }

    #[test]
    fn action_set_true_false() -> AnyResult<()> {
        let expected = Value::try_from(json!({
            "enable": true,
            "disable": false,
        }))?;
        println!("expected: {:?}", expected);

        let command = Command::new("test").args([
            Arg::new("enable").short('e').action(ArgAction::SetTrue),
            Arg::new("disable").short('d').action(ArgAction::SetFalse),
        ]);
        let args = ["test", "-e", "-d"];
        let conf = ConfigBuilder::load_one(ParserBuilder::new(command).args(args).build()?)?;
        let calculated = conf.get_value();
        println!("calculated: {:?}", calculated);
        assert_eq!(expected, *calculated);
        Ok(())
    }

    fn arg_types(args: &[&str], expected: Value, use_arg_types: bool) -> AnyResult<()> {
        let command = Command::new("test").args([
            Arg::new("age").short('a').value_parser(value_parser!(u32)),
            Arg::new("year")
                .short('y')
                .value_parser(value_parser!(String))
                .action(ArgAction::Append),
            Arg::new("month")
                .short('m')
                .value_parser(value_parser!(String)),
        ]);

        println!("expected: {:?}", expected);
        let conf = ConfigBuilder::load_one(
            ParserBuilder::new(command)
                .args(args)
                .use_arg_types(use_arg_types)
                .build()?,
        )?;
        let calculated = conf.get_value();
        println!("calculated: {:?}", calculated);
        assert_eq!(expected, *calculated);
        Ok(())
    }

    #[test]
    fn use_arg_types() -> AnyResult<()> {
        let expected = Value::try_from(json!({
            "age": 42,
            "year": ["2019", "2022"],
            "month": "1",
        }))?;
        println!("expected: {:?}", expected);

        let args = ["test", "-a", "42", "-y", "2019", "-y", "2022", "-m", "'1'"];
        arg_types(&args, expected, true)
    }

    #[test]
    fn not_use_arg_types() -> AnyResult<()> {
        let expected = Value::try_from(json!({
            "age": 42,
            "year": [2019, 2022],
            "month": "1",
        }))?;
        println!("expected: {:?}", expected);

        let args = [
            "test", "-a", "42", "-y", "2019", "-y", "2022", "-m", r#""1""#,
        ];
        arg_types(&args, expected, false)
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
                    "enabled": true
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
            "enabled": true
        }))?;

        user_add(expected, true)
    }

    #[test]
    fn not_use_defaults_for_bool() -> AnyResult<()> {
        let expected = Value::try_from(json!({
            "config": "config.toml"
        }))?;
        println!("expected: {:?}", expected);

        let command = Command::new("test").args([
            Arg::new("config").short('c'),
            Arg::new("enable").short('e').action(ArgAction::SetTrue),
        ]);
        let args = ["test", "-c", "config.toml"];
        let conf = ConfigBuilder::load_one(ParserBuilder::new(command).args(args).build()?)?;
        let calculated = conf.get_value();
        println!("calculated: {:?}", calculated);
        assert_eq!(expected, *calculated);
        Ok(())
    }

    #[test]
    fn use_defaults() -> AnyResult<()> {
        let expected = Value::try_from(json!({
            "config": "default.toml",
            "enable": false,
        }))?;
        println!("expected: {:?}", expected);

        let command = Command::new("test").args([
            Arg::new("config").short('c').default_value("default.toml"),
            Arg::new("enable").short('e').action(ArgAction::SetTrue),
        ]);
        let args = ["test"];
        let conf = ConfigBuilder::load_one(
            ParserBuilder::new(command)
                .args(args)
                .use_defaults(true)
                .build()?,
        )?;
        let calculated = conf.get_value();
        println!("calculated: {:?}", calculated);
        assert_eq!(expected, *calculated);
        Ok(())
    }

    fn not_use_defaults_for_string(explicit: bool) -> AnyResult<()> {
        let expected = Value::try_from(json!({
            "enable": true
        }))?;
        println!("expected: {:?}", expected);

        let config_arg = if explicit {
            Arg::new("config").short('c').default_value("default.toml")
        } else {
            Arg::new("config").short('c')
        };

        let command = Command::new("test").args([
            config_arg,
            Arg::new("enable").short('e').action(ArgAction::SetTrue),
        ]);
        let args = ["test", "-e"];
        let conf = ConfigBuilder::load_one(ParserBuilder::new(command).args(args).build()?)?;
        let calculated = conf.get_value();
        println!("calculated: {:?}", calculated);
        assert_eq!(expected, *calculated);
        Ok(())
    }

    #[test]
    fn not_use_defaults_implicit_for_string() -> AnyResult<()> {
        not_use_defaults_for_string(false)
    }

    #[test]
    fn not_use_defaults_explicit_for_string() -> AnyResult<()> {
        not_use_defaults_for_string(true)
    }
}
