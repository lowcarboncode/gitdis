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

#[derive(Debug, PartialEq, Clone)]
pub enum RefType {
    Object,
    Array,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Ref {
    pub ref_type: RefType,
    pub items: Vec<String>,
}

/// Transforms a nested `Value` into a flat `HashMap` with dot-separated keys.
/// This version also supports arrays, representing their indices in the keys.
/// Example:
/// Input: { "key": { "key2": ["value1", {"key3": "value2"}] } }
/// Output: { "key.key2.0": "value1", "key.key2.1.key3": "value2" }
pub fn value_to_mapper(value: Value) -> (Value, HashMap<String, Ref>) {
    fn flatten(
        value: &Value,
        prefix: String,
        mapper: &mut HashMap<String, Value>,
        refs: &mut HashMap<String, Ref>,
    ) {
        match value {
            Value::Object(obj) => {
                let mut list_keys = Vec::new();

                for (key, sub_value) in obj.iter() {
                    let new_key = if prefix.is_empty() {
                        key.to_string()
                    } else {
                        format!("{}.{}", prefix, key)
                    };

                    list_keys.push(new_key.clone());
                    flatten(sub_value, new_key, mapper, refs);
                }

                refs.insert(
                    prefix.to_string(),
                    Ref {
                        ref_type: RefType::Object,
                        items: list_keys,
                    },
                );
            }
            Value::Array(arr) => {
                let mut list_keys = Vec::new();

                for (index, sub_value) in arr.into_iter().enumerate() {
                    let new_key = if prefix.is_empty() {
                        index.to_string()
                    } else {
                        format!("{}.{}", prefix, index)
                    };

                    list_keys.push(new_key.clone());
                    flatten(sub_value, new_key, mapper, refs);
                }

                refs.insert(
                    prefix.to_string(),
                    Ref {
                        ref_type: RefType::Array,
                        items: list_keys,
                    },
                );
            }
            _ => {
                mapper.insert(prefix, value.clone());
            }
        }
    }

    let mut mapper = HashMap::new();
    let mut refs = HashMap::new();
    flatten(&value, String::new(), &mut mapper, &mut refs);
    (mapper.to_value(), refs)
}

pub fn to_value(key: String, file: &str, content: &str) -> (Value, HashMap<String, Ref>) {
    let value = {
        let mut value = HashMap::new();

        value.insert(key, parse_value(file, content));

        value.to_value()
    };

    value_to_mapper(value)
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

        let refer = value_to_mapper(value).1;

        assert_eq!(
            refer.get("key1.key2.key3.key4.key5.key6").unwrap(),
            &Ref {
                ref_type: RefType::Object,
                items: vec!["key1.key2.key3.key4.key5.key6.key7".to_string()]
            }
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

        let mapper = value_to_mapper(value).0;

        assert_eq!(
            mapper
                .get("key1.key2.key3.key4.key5.key6.key7")
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

        let refs = value_to_mapper(value).1;

        assert_eq!(
            refs.get("key1.key2.key3").unwrap(),
            &Ref {
                ref_type: RefType::Array,
                items: vec![
                    "key1.key2.key3.0".to_string(),
                    "key1.key2.key3.1".to_string()
                ]
            }
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

        let mapper = value_to_mapper(value).0;

        assert_eq!(mapper.get("key1.key2.key3.1").unwrap().as_string(), "world");
    }
}
