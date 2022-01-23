#[cfg(all(feature = "env", feature = "json", feature = "yaml", feature = "cmd"))]
mod integration {
    use clap::{App, Arg, ArgSettings};
    use irx_config::parsers::{cmd, env, json, yaml};
    use irx_config::{json, AnyResult, ConfigBuilder, MergeCase, Value};
    use std::env as StdEnv;

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

    fn format_option_bool(option: Option<bool>) -> String {
        if let Some(o) = option {
            format!("{}", o)
        } else {
            "None".to_string()
        }
    }

    fn format_option_merge_case(option: Option<MergeCase>) -> String {
        if let Some(o) = option {
            format!("{:?}", o)
        } else {
            "None".to_string()
        }
    }

    fn create_app<'a>() -> App<'a> {
        App::new("test").version("1.0").args([
            Arg::new("config")
                .short('c')
                .long("config")
                .setting(ArgSettings::TakesValue | ArgSettings::Required),
            Arg::new("settings:name")
                .short('n')
                .long("name")
                .setting(ArgSettings::TakesValue | ArgSettings::Required),
            Arg::new("logger:timeout").short('t').takes_value(true),
            Arg::new("verbose").short('v'),
        ])
    }

    #[test]
    fn full() -> AnyResult<()> {
        let mut expected = Value::try_from(json!({
            "logger": {
                "address": "json.localhost",
                "tag": "logger json file tag",
                "timeout": 1000
            },
            "settings": {
                "id": 42,
                "logger": {
                    "address": "yaml.localhost",
                    "tag": "logger yaml file tag"
                },
                "name": "Joe from cmd"
            },
            "verbose": 1
        }))?;
        expected.set_by_keys(["config"], resource_path!("config.json").to_owned())?;

        let args = [
            "test",
            "-c",
            resource_path!("config.json"),
            "-n",
            "Joe from cmd",
            "-t",
            "1000",
            "-v",
        ];
        let cmd_parser = cmd::ParserBuilder::new(create_app()).args(args).build()?;

        let json_path = resource_path!("config.json");
        let yaml_path = resource_path!("config.yaml");

        let json_parser = json::ParserBuilder::default()
            .default_path(json_path)
            .path_option("config")
            .build()?;

        let yaml_parser = yaml::ParserBuilder::default()
            .default_path(yaml_path)
            .build()?;

        let env_parser = env::ParserBuilder::default()
            .default_prefix("APP_")
            .prefix_option("prefix")
            .build()?;

        let config = ConfigBuilder::default()
            .append_parser(cmd_parser)
            .append_parser(json_parser)
            .append_parser(yaml_parser)
            .append_parser(env_parser)
            .load()?;
        println!("{}", config);
        assert_eq!(expected, *config.get_value());
        Ok(())
    }

    fn test_case<E, M>(expected: &Value, env_case: E, merge_case: M) -> AnyResult<()>
    where
        E: Into<Option<bool>> + Copy,
        M: Into<Option<MergeCase>> + Copy,
    {
        let json_path = resource_path!("config.json");
        let json_parser = json::ParserBuilder::default()
            .default_path(json_path)
            .path_option("json-config")
            .build()?;

        StdEnv::set_var("APP_SETTINGS__NAME", "name from env");
        StdEnv::set_var("APP_LOGGER__TAG", "logger env file tag");
        let mut builder = env::ParserBuilder::default();
        builder.default_prefix("APP_").prefix_option("prefix");
        if let Some(c) = env_case.into() {
            builder.case_sensitive(c);
        }
        let env_parser = builder.build()?;

        let mut builder = ConfigBuilder::default()
            .append_parser(json_parser)
            .append_parser(env_parser);
        if let Some(m) = merge_case.into() {
            builder = builder.merge_case(m)
        }
        let config = builder.load()?;
        println!("{}", config);
        assert_eq!(
            expected,
            config.get_value(),
            "env_case: {}, merge_case: {}",
            format_option_bool(env_case.into()),
            format_option_merge_case(merge_case.into())
        );
        Ok(())
    }

    #[test]
    fn all_merge_cases() -> AnyResult<()> {
        let expected_case_off = Value::try_from(json!({
            "logger": {
                "address": "json.localhost",
                "tag": "logger json file tag"
            },
            "settings": {
                "id": 42,
                "name": "node json from file"
            }
        }))
        .unwrap();

        let expected_case_on = Value::try_from(json!({
            "LOGGER": {
                "TAG": "logger env file tag"
            },
            "SETTINGS": {
                "NAME": "name from env"
            },
            "logger": {
                "address": "json.localhost",
                "tag": "logger json file tag"
            },
            "settings": {
                "id": 42,
                "name": "node json from file"
            }
        }))
        .unwrap();

        let tests_params = [
            (&expected_case_off, None, None),
            (&expected_case_on, Some(true), Some(MergeCase::Auto)),
            (&expected_case_on, Some(true), Some(MergeCase::Sensitive)),
            (&expected_case_off, Some(true), Some(MergeCase::Insensitive)),
            (&expected_case_off, Some(false), Some(MergeCase::Auto)),
            (&expected_case_off, Some(false), Some(MergeCase::Sensitive)),
            (
                &expected_case_off,
                Some(false),
                Some(MergeCase::Insensitive),
            ),
        ];

        for (expected, env_case, merge_case) in tests_params {
            test_case(expected, env_case, merge_case)?;
        }

        Ok(())
    }
}
