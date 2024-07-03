use super::{Clause, Condition, ConditionGroup, ConditionToken, LogicalOperator, Operator};

use pest::Parser;
use pest_derive::Parser;
use std::collections::HashMap;
use valu3::prelude::*;

#[derive(Parser)]
#[grammar = "condition/parser/json.pest"]
pub struct JsonParser;

pub fn parse_json_to_clause(json: &str) -> Result<Clause, String> {
    let pairs = JsonParser::parse(Rule::json, json.trim()).map_err(|e| e.to_string())?;
    parse_pairs_to_clause(pairs)
}

fn parse_pairs_to_clause(pairs: pest::iterators::Pairs<Rule>) -> Result<Clause, String> {
    let mut conditions = vec![];

    for pair in pairs {
        match pair.as_rule() {
            Rule::object => {
                let mut map = parse_object(pair.into_inner(), true)?;
                conditions.append(&mut map);
            }
            Rule::EOI => {
                // Do nothing
            }
            rule => return Err(format!("Unexpected rule in parse_pairs_to_clause: {:?}", rule)),
        }
    }

    Ok(Clause::ConditionGroup(ConditionGroup { conditions }))
}

fn parse_object(pairs: pest::iterators::Pairs<Rule>, initial: bool) -> Result<Vec<ConditionToken>, String> {
    let mut conditions = vec![];
    let mut not_logical = false;
    for pair in pairs {
        match pair.as_rule() {
            Rule::pair => {
                let mut inner = pair.clone().into_inner();
                let key = inner.next().unwrap().as_str().trim_matches('"').to_string();
                let value = inner.next().unwrap();

                match key.as_str() {
                    "$and" => {
                        let mut sub_conditions = parse_logical_operator(value.into_inner(), LogicalOperator::And, initial)?;
                        conditions.append(&mut sub_conditions);
                        not_logical = false;
                    }
                    "$or" => {
                        let mut sub_conditions = parse_logical_operator(value.into_inner(), LogicalOperator::Or, initial)?;

                        if not_logical {
                            conditions.push(ConditionToken::LogicalOperator(LogicalOperator::And));
                            conditions.push(ConditionToken::ConditionGroup(ConditionGroup {
                                conditions: sub_conditions,
                            }));
                        } else {
                            conditions.append(&mut sub_conditions);
                        }

                        conditions.append(&mut sub_conditions);
                        not_logical = false;
                    }
                    _ => {
                        if not_logical {
                            conditions.push(ConditionToken::LogicalOperator(LogicalOperator::And));
                        }

                        let condition = parse_condition(key, value)?;
                        conditions.push(ConditionToken::Condition(condition));

                        not_logical = true;
                    }
                }
            }
            rule => return Err(format!("Unexpected rule parse_object: {:?}", rule)),
        }
    }

    Ok(conditions)
}

fn parse_value(pair: pest::iterators::Pair<Rule>) -> Result<Value, String> {
    match pair.as_rule() {
        Rule::string => {
            let value = pair.as_str().trim_matches('"').to_string();
            Ok(value.to_value())
        }
        Rule::number => {
            let value = match Number::try_from(pair.as_str()) {
                Ok(value) => value,
                Err(e) => return Err(format!("Error parsing number: {:?}", e)),
            };

            Ok(Value::Number(value))
        }
        Rule::boolean => {
            let value = pair.as_str().parse::<bool>().unwrap();
            Ok(value.to_value())
        }
        Rule::null => Ok(Value::Null),
        Rule::array => {
            let mut values = vec![];

            for pair in pair.into_inner() {
                let value = parse_value(pair)?;
                values.push(value);
            }

            Ok(values.to_value())
        }
        Rule::object => {
            let mut map = HashMap::new();

            for pair in pair.into_inner() {
                match pair.as_rule() {
                    Rule::pair => {
                        let mut inner = pair.into_inner();
                        let key = inner.next().unwrap().as_str().trim_matches('"').to_string();
                        let re = inner.next().unwrap();
                        let value = parse_value(re).unwrap();

                        map.insert(key, value);
                    }
                    rule => return Err(format!("Unexpected rule parse_object_value: {:?}", rule)),
                }
            }

            Ok(map.to_value())
        }
        rule => Err(format!("Unexpected rule parse_value: {:?}", rule)),
    }
}

fn parse_logical_operator(pairs: pest::iterators::Pairs<Rule>, logical_operator: LogicalOperator, initial: bool) -> Result<Vec<ConditionToken>, String> {
    let mut conditions = vec![];

    for pair in pairs {
        match pair.as_rule() {
            Rule::object => {
                let mut map = parse_object(pair.into_inner(), false)?;
                conditions.append(&mut map);
            }
            rule => return Err(format!("Unexpected rule parse_array: {:?}", rule)),
        }
    }

    let first_condition = conditions.remove(0);
    let mut conditions_logical = vec![first_condition];

    for condition in conditions {
        conditions_logical.push(ConditionToken::LogicalOperator(logical_operator.clone()));
        conditions_logical.push(condition);
    }

    if !initial {
        Ok(vec![ConditionToken::ConditionGroup(ConditionGroup {
            conditions: conditions_logical,
        })])
    } else {
        Ok(conditions_logical)
    }
}

fn parse_condition(key: String, pair: pest::iterators::Pair<Rule>) -> Result<Condition, String> {
    match pair.as_rule() {
        Rule::object => {
            let mut inner = pair.clone().into_inner();
            let mut operator_pair = inner.next().unwrap().into_inner();
            let operator_key = operator_pair.next().unwrap().into_inner().as_str();

            let operator = match operator_key {
                "$eq" => Operator::Equal,
                "$ne" => Operator::NotEqual,
                "$gt" => Operator::GreaterThan,
                "$gte" => Operator::GreaterThanOrEqual,
                "$lt" => Operator::LessThan,
                "$lte" => Operator::LessThanOrEqual,
                "$in" => Operator::In,
                "$nin" => Operator::NotIn,
                "$regex" => Operator::Regex,
                "$notRegex" => Operator::NotRegex,
                "$like" => Operator::Like,
                "$notLike" => Operator::NotLike,
                "$between" => Operator::Between,
                "$notBetween" => Operator::NotBetween,
                _ => {
                    let value = parse_value(pair.clone())?;
                    return Ok(Condition {
                        operator: Operator::Equal,
                        left: key.to_value(),
                        right: value,
                    });
                }
            };

            let right = operator_pair
                .next()
                .map(|p| p.as_str().trim_matches('"').to_string());

            Ok(Condition {
                operator,
                left: key.to_value(),
                right: right.to_value(),
            })
        }
        Rule::string | Rule::number | Rule::boolean | Rule::null | Rule::array => {
            Ok(Condition {
                operator: Operator::Equal,
                left: key.to_value(),
                right: parse_value(pair)?,
            })
        }
        rule => return Err(format!("Unexpected rule parse_condition: {:?}", rule)),
    }
}

#[cfg(test)]
mod tests {
    use crate::valu3::types::object::Object;
    use super::*;

    #[test]
    fn test_parse_json_to_clause_and() {
        let json = r#"{
            "$and": [
                {"name": {"$eq": "John"}},
                {"age": {"$gt": 18}}
            ]
        }"#;

        let clause = parse_json_to_clause(json).unwrap();

        let expected_clause = Clause::ConditionGroup(ConditionGroup {
            conditions: vec![
                ConditionToken::Condition(Condition {
                    operator: Operator::Equal,
                    left: "name".to_value(),
                    right: Some("John").to_value(),
                }),
                ConditionToken::LogicalOperator(LogicalOperator::And),
                ConditionToken::Condition(Condition {
                    operator: Operator::GreaterThan,
                    left: "age".to_value(),
                    right: Some("18").to_value(),
                }),
            ],
        });

        assert_eq!(clause, expected_clause);
    }

    #[test]
    fn test_parse_json_to_clause_or() {
        let json = r#"{
            "$or": [
                {"name": {"$eq": "John"}},
                {"age": {"$gt": 18}}
            ]
        }"#;

        let clause = parse_json_to_clause(json).unwrap();

        let expected_clause = Clause::ConditionGroup(ConditionGroup {
            conditions: vec![
                ConditionToken::Condition(Condition {
                    operator: Operator::Equal,
                    left: "name".to_value(),
                    right: Some("John").to_value(),
                }),
                ConditionToken::LogicalOperator(LogicalOperator::Or),
                ConditionToken::Condition(Condition {
                    operator: Operator::GreaterThan,
                    left: "age".to_value(),
                    right: Some("18").to_value(),
                }),
            ],
        });

        assert_eq!(clause, expected_clause);
    }

    #[test]
    fn test_parse_json_to_clause_and_and_or() {
        let json = r#"{
            "$and": [
                {"name": {"$eq": "John"}},
                {"$or": [
                    {"age": {"$gt": 18}},
                    {"age": {"$lt": 30}}
                ]}
            ]
        }"#;

        let clause = parse_json_to_clause(json).unwrap();

        let expected_clause = Clause::ConditionGroup(ConditionGroup {
            conditions: vec![
                ConditionToken::Condition(Condition {
                    operator: Operator::Equal,
                    left: "name".to_value(),
                    right: Some("John").to_value(),
                }),
                ConditionToken::LogicalOperator(LogicalOperator::And),
                ConditionToken::ConditionGroup(ConditionGroup {
                    conditions: vec![
                        ConditionToken::Condition(Condition {
                            operator: Operator::GreaterThan,
                            left: "age".to_value(),
                            right: Some("18").to_value(),
                        }),
                        ConditionToken::LogicalOperator(LogicalOperator::Or),
                        ConditionToken::Condition(Condition {
                            operator: Operator::LessThan,
                            left: "age".to_value(),
                            right: Some("30").to_value(),
                        }),
                    ],
                }),
            ],
        });

        assert_eq!(clause, expected_clause);
    }

    #[test]
    fn test_parse_json_to_clause_complex() {
        let json = r#"
        {
            "$and": [
                {"name": {"$eq": "John"}},
                {"$or": [
                    {"age": {"$gt": 18}},
                    {"age": {"$lt": 30}}
                ]},
                {"$and": [
                    {
                        "name": {"$like": "John"}
                    },
                    {
                        "age": {"$regex": "^[0-9]*$"}
                    },
                    {"$or": [
                        {"age": {"$notRegex": 18}},
                        {"age": {"$lt": 30}}
                    ]}
                ]}
            ]
        }
        "#;

        let clause = parse_json_to_clause(json).unwrap();

        let expected_clause = Clause::ConditionGroup(ConditionGroup {
            conditions: vec![
                ConditionToken::Condition(Condition {
                    operator: Operator::Equal,
                    left: "name".to_value(),
                    right: Some("John").to_value(),
                }),
                ConditionToken::LogicalOperator(LogicalOperator::And),
                ConditionToken::ConditionGroup(ConditionGroup {
                    conditions: vec![
                        ConditionToken::Condition(Condition {
                            operator: Operator::GreaterThan,
                            left: "age".to_value(),
                            right: Some("18").to_value(),
                        }),
                        ConditionToken::LogicalOperator(LogicalOperator::Or),
                        ConditionToken::Condition(Condition {
                            operator: Operator::LessThan,
                            left: "age".to_value(),
                            right: Some("30").to_value(),
                        }),
                    ],
                }),
                ConditionToken::LogicalOperator(LogicalOperator::And),
                ConditionToken::ConditionGroup(ConditionGroup {
                    conditions: vec![
                        ConditionToken::Condition(Condition {
                            operator: Operator::Like,
                            left: "name".to_value(),
                            right: Some("John").to_value(),
                        }),
                        ConditionToken::LogicalOperator(LogicalOperator::And),
                        ConditionToken::Condition(Condition {
                            operator: Operator::Regex,
                            left: "age".to_value(),
                            right: Some("^[0-9]*$").to_value(),
                        }),
                        ConditionToken::LogicalOperator(LogicalOperator::And),
                        ConditionToken::ConditionGroup(ConditionGroup {
                            conditions: vec![
                                ConditionToken::Condition(Condition {
                                    operator: Operator::NotRegex,
                                    left: "age".to_value(),
                                    right: Some("18").to_value(),
                                }),
                                ConditionToken::LogicalOperator(LogicalOperator::Or),
                                ConditionToken::Condition(Condition {
                                    operator: Operator::LessThan,
                                    left: "age".to_value(),
                                    right: Some("30").to_value(),
                                }),
                            ],
                        }),
                    ],
                }),
            ],
        });

        assert_eq!(clause, expected_clause);
    }


    #[test]
    fn test_parse_json_to_clause_default_equal() {
        let json = r#"{
            "name": "John",
            "items": ["apple", "banana"],
            "list": {
                "a": 1,
                "b": 2
            },
            "$or": [
                {"age": 18},
                {"age": 30}
            ]
        }"#;

        let clause = parse_json_to_clause(json).unwrap();

        let expected_clause = Clause::ConditionGroup(ConditionGroup {
            conditions: vec![
                ConditionToken::Condition(Condition {
                    operator: Operator::Equal,
                    left: "name".to_value(),
                    right: Some("John").to_value(),
                }),
                ConditionToken::LogicalOperator(LogicalOperator::And),
                ConditionToken::Condition(Condition {
                    operator: Operator::Equal,
                    left: "items".to_value(),
                    right: Some(vec!["apple", "banana"]).to_value(),
                }),
                ConditionToken::LogicalOperator(LogicalOperator::And),
                ConditionToken::Condition(Condition {
                    operator: Operator::Equal,
                    left: "list".to_value(),
                    right: Some(Object::from(vec![("a", 1), ("b", 2)])).to_value(),
                }),
                ConditionToken::LogicalOperator(LogicalOperator::And),
                ConditionToken::ConditionGroup(ConditionGroup {
                    conditions: vec![
                        ConditionToken::Condition(Condition {
                            operator: Operator::Equal,
                            left: "age".to_value(),
                            right: Some(18).to_value(),
                        }),
                        ConditionToken::LogicalOperator(LogicalOperator::Or),
                        ConditionToken::Condition(Condition {
                            operator: Operator::Equal,
                            left: "age".to_value(),
                            right: Some(30).to_value(),
                        }),
                    ],
                }),
            ],
        });

        assert_eq!(clause, expected_clause);
    }
}
