use std::collections::HashMap;

use quickleaf::{prelude::ToValueBehavior, Value};

const EXT_JSON: &str = ".json";
const EXT_YML: &str = ".yml";
const EXT_YAML: &str = ".yaml";

pub fn parse_value(file: &str, content: &str) -> Value {
    if file.ends_with(EXT_JSON) {
        Value::json_to_value(&content).unwrap_or(Value::Undefined)
    } else if file.ends_with(EXT_YML) || file.ends_with(EXT_YAML) {
        serde_yaml::from_str(&content).unwrap_or(Value::Undefined)
    } else {
        Value::Undefined
    }
}

/// Transforms a nested `Value` into a flat `HashMap` with dot-separated keys.
/// This version also supports arrays, representing their indices in the keys.
/// Example:
/// Input: { "key": { "key2": ["value1", {"key3": "value2"}] } }
/// Output: { "key.key2.0": "value1", "key.key2.1.key3": "value2" }
pub fn value_to_mapper(default_prefix: &str, value: Value) -> HashMap<String, Value> {
    fn flatten(value: &Value, prefix: String, mapper: &mut HashMap<String, Value>) {
        match value {
            Value::Object(obj) => {
                for (key, sub_value) in obj.iter() {
                    let new_key = if prefix.is_empty() {
                        key.to_string()
                    } else {
                        format!("{}.{}", prefix, key)
                    };
                    flatten(sub_value, new_key, mapper);
                }

                mapper.insert(prefix.to_string(), obj.to_value());
            }
            Value::Array(arr) => {
                for (index, sub_value) in arr.into_iter().enumerate() {
                    let new_key = if prefix.is_empty() {
                        index.to_string()
                    } else {
                        format!("{}.{}", prefix, index)
                    };
                    flatten(sub_value, new_key, mapper);
                }

                mapper.insert(prefix.to_string(), arr.to_value());
            }
            _ => {
                mapper.insert(prefix, value.clone());
            }
        }
    }

    let mut mapper = HashMap::new();
    flatten(&value, default_prefix.to_string(), &mut mapper);
    mapper
}

pub fn to_value(key: &str, file: &str, content: &str) -> Value {
    let value = parse_value(file, content);
    let mapper = value_to_mapper(key, value).to_value();
    mapper
}

pub fn is_valid_file(path: &str) -> bool {
    path.ends_with(EXT_JSON) || path.ends_with(EXT_YML) || path.ends_with(EXT_YAML)
}

#[cfg(test)]
mod tests {
    use quickleaf::prelude::StringBehavior;

    use super::*;

    #[test]
    fn test_value_to_mapper_object() {
        let value = Value::json_to_value(
            r#"
            {
                "key1": {
                    "key2": {
                        "key3": {
                            "key4": {
                                "key5": {
                                    "key6": {
                                        "key7": "hello"
                                    }
                                }
                            }
                        }
                    }
                }
            }
            "#,
        )
        .unwrap();

        let mapper = value_to_mapper("data.content", value);

        assert_eq!(
            mapper
                .get("data.content.key1.key2.key3.key4.key5.key6")
                .unwrap(),
            &Value::from({
                let mut map = HashMap::new();
                map.insert("key7", "hello".to_string());
                map
            })
        );
    }

    #[test]
    fn test_value_to_mapper_object_value() {
        let value = Value::json_to_value(
            r#"
            {
                "key1": {
                    "key2": {
                        "key3": {
                            "key4": {
                                "key5": {
                                    "key6": {
                                        "key7": "hello"
                                    }
                                }
                            }
                        }
                    }
                }
            }
            "#,
        )
        .unwrap();

        let mapper = value_to_mapper("data.content", value);

        assert_eq!(
            mapper
                .get("data.content.key1.key2.key3.key4.key5.key6.key7")
                .unwrap()
                .as_string(),
            "hello"
        );
    }

    #[test]
    fn test_value_to_mapper_list() {
        let value = Value::json_to_value(
            r#"
            {
                "key1": {
                    "key2": {
                        "key3": ["hello", "world"]
                    }
                }
            }
            "#,
        )
        .unwrap();

        let mapper = value_to_mapper("data.content", value);

        assert_eq!(
            mapper.get("data.content.key1.key2.key3").unwrap(),
            &vec![Value::from("hello"), Value::from("world")].to_value()
        );
    }

    #[test]
    fn test_value_to_mapper_list_item() {
        let value = Value::json_to_value(
            r#"
            {
                "key1": {
                    "key2": {
                        "key3": ["hello", "world"]
                    }
                }
            }
            "#,
        )
        .unwrap();

        let mapper = value_to_mapper("data.content", value);

        assert_eq!(
            mapper
                .get("data.content.key1.key2.key3.1")
                .unwrap()
                .as_string(),
            "world"
        );
    }
}
