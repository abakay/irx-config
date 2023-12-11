use crate::{json, value::SealedState, AnyResult, Case, ConfigBuilder, Parse, Value};
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Section {
    pub id: u32,
    pub name: String,
    pub logger: String,
    pub tag: String,
}

#[derive(Deserialize, Debug)]
pub struct Sections {
    pub settings: Section,
}

pub struct JsonStringParser {
    data: String,
}

impl JsonStringParser {
    pub fn new(data: impl Into<String>) -> Self {
        JsonStringParser { data: data.into() }
    }
}

impl Case for JsonStringParser {}

impl Parse for JsonStringParser {
    fn parse(&mut self, _: &Value) -> AnyResult<Value> {
        Ok(serde_json::from_str(&self.data)?)
    }
}

pub struct ValueParser {
    data: Value,
}

impl ValueParser {
    pub fn new(data: Value) -> Self {
        ValueParser { data }
    }
}

impl Case for ValueParser {}

impl Parse for ValueParser {
    fn parse(&mut self, _value: &Value) -> AnyResult<Value> {
        Ok(self.data.clone())
    }
}

#[derive(Deserialize, Debug)]
#[allow(dead_code)] // For `Rust 1.57.0`
struct Person {
    name: String,
    age: u8,
    phones: Vec<String>,
}

const SETTINGS_FIRST: &str = r#"
{
    "settings": {
        "logger": "from first"
    }
}
"#;

const SETTINGS_SECOND: &str = r#"
{
    "settings": {
        "id": 2,
        "name": "node-2",
        "logger": "from second"
    },
    "connections": {
        "node-1": "tcp://node-1",
        "node-2": "tcp://node-2"
    }
}
"#;

const SETTINGS_THIRD: &str = r#"
{
    "settings": {
        "id": 3,
        "tag": "from third",
        "extra": []
    }
}
"#;

mod config {
    use super::*;

    #[test]
    fn single_parser() -> AnyResult<()> {
        let expected = Person {
            name: "John Doe".to_owned(),
            age: 43,
            phones: vec!["+44 1234567".to_owned(), "+44 2345678".to_owned()],
        };

        let data = Value::try_from(json!(
        {
            "name": "John Doe",
            "age": 43,
            "phones": [
                "+44 1234567",
                "+44 2345678"
            ]
        }))?;

        let conf = ConfigBuilder::load_one(ValueParser::new(data))?;
        let person: Person = conf.get()?;

        println!("Person: {person:?}");
        assert_eq!(format!("{expected:?}"), format!("{person:?}"));
        Ok(())
    }

    #[test]
    fn merge_nested_structs() -> AnyResult<()> {
        let parsers = [
            JsonStringParser::new(SETTINGS_FIRST),
            JsonStringParser::new(SETTINGS_SECOND),
            JsonStringParser::new(SETTINGS_THIRD),
        ];
        let conf = ConfigBuilder::load_from(parsers)?;
        let sections: Sections = conf.get()?;

        let s = Sections {
            settings: Section {
                id: 2,
                name: "node-2".to_owned(),
                logger: "from first".to_owned(),
                tag: "from third".to_owned(),
            },
        };
        println!("{sections:?}");
        assert_eq!(format!("{s:?}"), format!("{sections:?}"));
        Ok(())
    }

    #[test]
    #[should_panic(expected = "found while parsing")]
    fn parse_error() {
        let data = r#"
        {
            "settings": {
                "logger": "from first
            }
        }
        "#;
        let conf = ConfigBuilder::load_one(JsonStringParser::new(data)).unwrap();
        let value: Value = conf.get().unwrap();
        println!("{value:?}");
    }

    #[test]
    fn get_by_keys_none() -> AnyResult<()> {
        let conf = ConfigBuilder::load_one(JsonStringParser::new(SETTINGS_SECOND))?;
        let dummy: Option<u32> = conf.get_by_keys(["a", "b"])?;
        assert_eq!(None, dummy);
        Ok(())
    }

    #[test]
    fn get_by_keys_empty() -> AnyResult<()> {
        let conf = ConfigBuilder::load_one(JsonStringParser::new(SETTINGS_SECOND))?;
        let expected = conf.get_value();
        println!("expected: {expected:?}");
        assert_eq!(*expected, conf.get_by_keys([""; 0])?.unwrap());
        Ok(())
    }

    #[test]
    fn get_by_key_path_empty() -> AnyResult<()> {
        let conf = ConfigBuilder::load_one(JsonStringParser::new(SETTINGS_SECOND))?;
        let expected = conf.get_value();
        println!("expected: {expected:?}");
        assert_eq!(*expected, conf.get_by_key_path("")?.unwrap());
        Ok(())
    }

    #[test]
    fn get_by_key_path_none() -> AnyResult<()> {
        let conf = ConfigBuilder::load_one(JsonStringParser::new(SETTINGS_SECOND))?;
        assert_eq!(None, conf.get_by_key_path::<u32, _>(":")?);
        Ok(())
    }

    #[test]
    #[should_panic(expected = "value: EmptySeparator")]
    fn get_by_key_path_empty_sep() {
        let conf = ConfigBuilder::load_one(JsonStringParser::new(SETTINGS_SECOND)).unwrap();
        conf.get_by_key_path_with_delim::<u32, _, _>("connections", "")
            .unwrap();
    }

    #[test]
    fn get_by_key_path_one_level() -> AnyResult<()> {
        let expected = Value::try_from(json!(
        {
            "node-1": "tcp://node-1",
            "node-2": "tcp://node-2"
        }))?;

        let conf = ConfigBuilder::load_one(JsonStringParser::new(SETTINGS_SECOND))?;
        assert_eq!(expected, conf.get_by_key_path("connections")?.unwrap());
        Ok(())
    }

    #[test]
    fn get_by_key_path() -> AnyResult<()> {
        let expected = Value::try_from(json!("tcp://node-1"))?;
        let conf = ConfigBuilder::load_one(JsonStringParser::new(SETTINGS_SECOND))?;
        assert_eq!(
            expected,
            conf.get_by_key_path_with_delim::<Value, _, _>("connections/node-1", "/")?
                .unwrap()
        );
        Ok(())
    }

    #[test]
    fn get_by_key_path_longer_path() -> AnyResult<()> {
        let conf = ConfigBuilder::load_one(JsonStringParser::new(SETTINGS_SECOND))?;
        let path = "connections:node-1:box:instance".to_owned();
        assert_eq!(None, conf.get_by_key_path::<Value, _>(path)?);
        Ok(())
    }

    #[test]
    #[should_panic(expected = "invalid type: string")]
    fn type_mismatch() {
        let conf = ConfigBuilder::load_one(JsonStringParser::new(SETTINGS_SECOND)).unwrap();
        conf.get_by_key_path::<u32, _>("settings:name").unwrap();
    }

    #[test]
    fn display() -> AnyResult<()> {
        let expected = r#"Config: BLAKE3: 491cb76797a37492c3b10bbf93278b7ba568341f2f9237ba7f289956a24e1eac
{
  "connections": {
    "node-1": "tcp://node-1",
    "node-2": "tcp://node-2"
  },
  "settings": {
    "id": 2,
    "logger": "from second",
    "name": "node-2"
  }
}"#;
        let conf = ConfigBuilder::load_one(JsonStringParser::new(SETTINGS_SECOND))?;
        let conf = conf.to_string();
        println!("{conf}");
        assert_eq!(expected, conf);
        Ok(())
    }

    #[test]
    fn display_sealed() -> AnyResult<()> {
        let expected = r#"Config: BLAKE3: 0102dcddd8073a13d974e538331910072c0dd35bef692b291dca68479c267f40
{
  "perm": {
    "password": "********",
    "user": "jdoe"
  },
  "settings": {
    "id": 42,
    "name": "John Doe"
  }
}"#;
        let value = Value::try_from(json!({
            "perm": {
                "user": "jdoe",
                "password_sealed_": "secret"
            },
            "settings": {
                "id": 42,
                "name": "John Doe"
            }
        }))?;
        let exp_value = json!({
            "perm": {
                "user": "jdoe",
                "password": "secret"
            },
            "settings": {
                "id": 42,
                "name": "John Doe"
            }
        });
        let conf = ConfigBuilder::default()
            .sealed_suffix("_sealed_")
            .append_parser(ValueParser::new(value))
            .load()?;
        let conf_str = serde_json::to_string_pretty(&conf.get_value())?;
        println!("{conf_str}");
        assert_eq!(serde_json::to_string_pretty(&exp_value)?, conf_str);
        let conf = conf.to_string();
        println!("{conf}");
        assert_eq!(expected, conf);
        Ok(())
    }

    #[test]
    fn display_debug() -> AnyResult<()> {
        let expected = r#"Config { parsers: size(1), value: Value { value: Object {"connections": Object {"node-1": String("tcp://node-1"), "node-2": String("tcp://node-2")}, "settings": Object {"id": Number(2), "logger": String("from second"), "name": String("node-2")}}, sealed: None, sealed_state: On, case_on: true }, case_on: true, hash: Hash("491cb76797a37492c3b10bbf93278b7ba568341f2f9237ba7f289956a24e1eac"), sealed_suffix: "", keys_delimiter: ":" }"#;
        let conf = ConfigBuilder::load_one(JsonStringParser::new(SETTINGS_SECOND))?;
        println!("{conf:?}");
        assert_eq!(expected, format!("{conf:?}"));
        Ok(())
    }

    #[test]
    fn display_sealed_nested() -> AnyResult<()> {
        let expected = r#"Config: BLAKE3: 0102dcddd8073a13d974e538331910072c0dd35bef692b291dca68479c267f40
{
  "perm": "********",
  "settings": {
    "id": 42,
    "name": "John Doe"
  }
}"#;
        let value = Value::try_from(json!({
            "perm_sealed_": {
                "user": "jdoe",
                "password_sealed_": "secret"
            },
            "settings": {
                "id": 42,
                "name": "John Doe"
            }
        }))?;
        let exp_value = json!({
            "perm": {
                "user": "jdoe",
                "password": "secret"
            },
            "settings": {
                "id": 42,
                "name": "John Doe"
            }
        });
        let conf = ConfigBuilder::default()
            .sealed_suffix("_sealed_")
            .append_parser(ValueParser::new(value))
            .load()?;
        let conf_str = serde_json::to_string_pretty(&conf.get_value())?;
        println!("{conf_str}");
        assert_eq!(serde_json::to_string_pretty(&exp_value)?, conf_str);
        let conf = conf.to_string();
        println!("{conf}");
        assert_eq!(expected, conf);
        Ok(())
    }

    #[test]
    fn config_eq() -> AnyResult<()> {
        let value = Value::try_from(json!({
            "person": {
                "name": "John Doe",
                "age": 42
            }
        }))?;

        let conf_1 = ConfigBuilder::load_one(ValueParser::new(value.clone()))?;
        let conf_2 = ConfigBuilder::load_one(ValueParser::new(value))?;
        assert_eq!(conf_1, conf_2);
        Ok(())
    }

    #[test]
    fn config_ne() -> AnyResult<()> {
        let value_1 = Value::try_from(json!({
            "person": {
                "name": "John Doe",
                "age": 42
            }
        }))?;

        let value_2 = Value::try_from(json!({
            "person": {
                "name": "John Doe",
                "age": 24
            }
        }))?;

        let conf_1 = ConfigBuilder::load_one(ValueParser::new(value_1))?;
        let conf_2 = ConfigBuilder::load_one(ValueParser::new(value_2))?;
        assert_ne!(conf_1, conf_2);
        Ok(())
    }
}

mod value {
    use super::*;

    #[test]
    fn merge_with_unseal() -> AnyResult<()> {
        let mut value = Value::try_from(json!({
            "password_sealed_": "secret"
        }))?;
        value.seal("_sealed_");

        let dummy = Value::try_from(json!({
            "user": "name"
        }))?;

        value = value.merge(&dummy);
        println!("value: {value:?}");

        assert!(!value.is_sealed());
        Ok(())
    }

    #[test]
    fn merge_without_unseal() -> AnyResult<()> {
        let mut value = Value::try_from(json!({
            "password_sealed_": "secret"
        }))?;
        value.seal("_sealed_");

        let dummy = Value::try_from(json!(""))?;

        value = value.merge(&dummy);
        println!("value: {value:?}");

        assert!(value.is_sealed());
        Ok(())
    }

    #[test]
    fn merge_value() -> AnyResult<()> {
        let expected = Value::try_from(json!({
            "name": "John Doe",
            "age": 43,
            "phones": [
                "+44 1234567",
                "+44 2345678"
            ]
        }))?;

        let mut person = Value::try_from(json!({
            "name": "John Doe",
            "age": 43
        }))?;

        let phones = Value::try_from(json!({
            "phones": [
                "+44 1234567",
                "+44 2345678"
            ]
        }))?;

        person = person.merge(&phones);
        println!("person: {person}");
        assert_eq!(expected, person);
        Ok(())
    }

    #[test]
    fn set_by_key_path_to_empty_map() -> AnyResult<()> {
        let value = 42;
        let path = "path:to:number";
        let mut map = Value::default();
        let prev = map.set_by_key_path(path, value)?;
        assert_eq!(None, prev);
        assert_eq!(Some(value), map.get_by_key_path(path)?);
        Ok(())
    }

    #[test]
    fn set_by_path_to_map() -> AnyResult<()> {
        let mut map: Value = serde_json::from_str(SETTINGS_FIRST)?;
        let path = "settings:timeout";
        let value = 2.0;
        let prev = map.set_by_key_path(path, value)?;
        assert_eq!(None, prev);
        assert_eq!(Some(value), map.get_by_key_path(path)?);
        Ok(())
    }

    #[test]
    fn set_by_key_path_to_map_override() -> AnyResult<()> {
        let mut map: Value = serde_json::from_str(SETTINGS_FIRST)?;
        let path = "settings:logger";
        let value = "override".to_owned();
        let prev = map.set_by_key_path(path, value.clone())?.unwrap();
        assert_eq!(Value::try_from("from first")?, prev);
        assert_eq!(Some(value), map.get_by_key_path(path)?);
        Ok(())
    }

    #[test]
    fn set_by_empty_key_path() -> AnyResult<()> {
        let mut expected = Value::try_from(json!({
            "name": "john",
            "password_secret_": "password"
        }))?;
        expected.seal("_secret_");

        let mut value = expected.clone();
        let prev = value.set_by_key_path("", 42)?.unwrap();
        let de_value: i32 = value.get()?;

        println!("value: {value:?}");
        println!("value(deserialized): {de_value:?}");
        println!("prev: {prev:?}");

        assert_eq!(SealedState::Mutated, value.sealed_state());
        assert!(prev.is_sealed());
        assert_eq!(expected, prev);
        assert_eq!(42, de_value);
        Ok(())
    }

    #[test]
    #[should_panic(expected = "NotMap")]
    fn set_by_key_path_to_not_map() {
        let mut map: Value = serde_json::from_str(SETTINGS_FIRST).unwrap();
        let path = "settings:logger:id";
        let value = 42;
        map.set_by_key_path_with_delim(path, ":", value).unwrap();
    }

    #[test]
    fn reset_by_keys() -> AnyResult<()> {
        let expected = "localhost".to_string();
        let mut value = Value::default();
        value.set_by_key_path("settings:logger", expected.clone())?;
        let prev: String = value
            .set_by_keys(["settings", "logger"], 42)?
            .unwrap()
            .get()?;
        assert_eq!(expected, prev);
        Ok(())
    }

    #[test]
    fn display_sealed_none() -> AnyResult<()> {
        let value = Value::try_from(json!({
            "id": 42
        }))?;

        let expected = r#"{
  "id": 42
}"#;
        let value = format!("{value}");
        println!("value: {value}");
        assert_eq!(expected, value);
        Ok(())
    }

    #[test]
    fn display_sealed_off() -> AnyResult<()> {
        let mut value = Value::default();
        value.seal("").set_by_key_path("id", 42)?;
        let expected = r"{}";
        let value = format!("{value}");
        println!("value: {value}");
        assert_eq!(expected, value);
        Ok(())
    }

    #[test]
    fn display_debug_sealed_mutated() -> AnyResult<()> {
        let mut value = Value::try_from(json!({
            "perm": {
                "user": "jdoe",
                "password_sealed_": "secret"
            },
            "settings": {
                "id": 42,
                "name": "John Doe"
            }
        }))?;
        value.seal("_sealed_").set_by_key_path("settings:id", 42)?;

        let expected =
            r#"Value { value: Object {}, sealed: None, sealed_state: Mutated, case_on: true }"#;
        let value = format!("{value:?}");
        println!("value: {value}");
        assert_eq!(expected, value);
        Ok(())
    }
}
