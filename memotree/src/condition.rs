use regex::Regex;
use std::fmt::{self, Display, Formatter};
use valu3::prelude::*;

#[derive(ToValue, FromValue, Clone)]
pub enum Operator {
    Equal,
    NotEqual,
    GreaterThan,
    GreaterThanOrEqual,
    LessThan,
    LessThanOrEqual,
    Like,
    NotLike,
    In,
    NotIn,
    Between,
    NotBetween,
    IsNull,
    IsNotNull,
    Regex,
    NotRegex,
}

#[derive(ToValue, FromValue, Clone, PartialEq)]
pub enum LogicalOperator {
    And,
    Or,
}

#[derive(Clone, FromValue, ToValue)]
pub struct Condition {
    pub operator: Operator,
    pub left: Value,
    pub right: Value,
}

impl Condition {
    pub fn new<L, R>(operator: Operator, left: L, right: R) -> Self
    where
        L: Into<Value>,
        R: Into<Value>,
    {
        Self {
            operator,
            left: left.into(),
            right: right.into(),
        }
    }
}

#[derive(Clone)]
pub enum ConditionToken {
    Condition(Condition),
    LogicalOperator(LogicalOperator),
    ConditionGroup(ConditionGroup),
}

impl PrimitiveType for ConditionToken {}

impl FromValueBehavior for ConditionToken {
    type Item = Self;

    fn from_value(value: Value) -> Option<Self::Item> {
        match value.as_str() {
            "And" => Some(ConditionToken::LogicalOperator(LogicalOperator::And)),
            "Or" => Some(ConditionToken::LogicalOperator(LogicalOperator::Or)),
            _ => {
                let condition = Condition::from_value(value)?;
                Some(ConditionToken::Condition(condition))
            }
        }
    }
}

impl ToValueBehavior for ConditionToken {
    fn to_value(&self) -> Value {
        match self {
            ConditionToken::Condition(condition) => condition.to_value(),
            ConditionToken::LogicalOperator(operator) => operator.to_value(),
            ConditionToken::ConditionGroup(condition) => condition.to_value(),
        }
    }
}

#[derive(Clone, FromValue, ToValue)]
pub struct ConditionGroup {
    pub conditions: Vec<ConditionToken>,
}

pub enum Clause {
    ConditionGroup(ConditionGroup),
    Condition(Condition),
}

#[derive(Debug)]
pub enum Error {
    LeftConditionNotFound,
    RightConditionNotFound,
    LeftConditionNotString,
    RightConditionNotString,
    ConditionVariableNotFound,
    BetweenConditionInvalid,
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Error::LeftConditionNotFound => write!(f, "Left condition not found"),
            Error::RightConditionNotFound => write!(f, "Right condition not found"),
            Error::LeftConditionNotString => write!(f, "Left condition not string"),
            Error::RightConditionNotString => write!(f, "Right condition not string"),
            Error::ConditionVariableNotFound => write!(f, "Condition variable not found"),
            Error::BetweenConditionInvalid => write!(f, "Between condition invalid"),
        }
    }
}

impl Clause {
    pub fn condition<L, R>(operator: Operator, left: L, right: R) -> Self
    where
        L: Into<Value>,
        R: Into<Value>,
    {
        Self::Condition(Condition::new(operator, left, right))
    }

    pub fn group(conditions: Vec<ConditionToken>) -> Self {
        Self::ConditionGroup(ConditionGroup { conditions })
    }

    pub fn execute(&self, value: &Value) -> Result<bool, Error> {
        match self {
            Clause::ConditionGroup(condition_group) => {
                Self::execute_condition_group(condition_group, value)
            }
            Clause::Condition(condition) => Self::execute_condition(condition.clone(), value),
        }
    }

    fn execute_condition_group(
        condition_group: &ConditionGroup,
        value: &Value,
    ) -> Result<bool, Error> {
        let mut result = false;

        for condition in &condition_group.conditions {
            match condition {
                ConditionToken::Condition(condition) => {
                    result = Self::execute_condition(condition.clone(), value)?;
                }
                ConditionToken::LogicalOperator(operator) => match operator {
                    LogicalOperator::And => {
                        if !result {
                            return Ok(false);
                        } else {
                            result = false;
                        }
                    }
                    LogicalOperator::Or => {}
                },
                ConditionToken::ConditionGroup(condition_group) => {
                    result = Self::execute_condition_group(condition_group, value)?;
                }
            }
        }

        Ok(result)
    }

    pub fn execute_condition(condition: Condition, value: &Value) -> Result<bool, Error> {
        let value_left = match Self::resolve_condition_variable(&condition.left.to_value(), &value)
        {
            Ok(val) => val,
            Err(_) => return Err(Error::LeftConditionNotFound),
        };

        let value_right =
            match Self::resolve_condition_variable(&condition.right.to_value(), &value) {
                Ok(val) => val,
                Err(_) => return Err(Error::RightConditionNotFound),
            };

        match condition.operator {
            Operator::Equal => {
                if value_left.eq(&value_right) {
                    Ok(true)
                } else {
                    Ok(false)
                }
            }
            Operator::NotEqual => {
                if value_left.ne(&value_right) {
                    Ok(true)
                } else {
                    Ok(false)
                }
            }
            Operator::GreaterThan => {
                if value_left.gt(&value_right) {
                    Ok(true)
                } else {
                    Ok(false)
                }
            }
            Operator::GreaterThanOrEqual => {
                if value_left.ge(&value_right) {
                    Ok(true)
                } else {
                    Ok(false)
                }
            }
            Operator::LessThan => {
                if value_left.lt(&value_right) {
                    Ok(true)
                } else {
                    Ok(false)
                }
            }
            Operator::LessThanOrEqual => {
                if value_left.le(&value_right) {
                    Ok(true)
                } else {
                    Ok(false)
                }
            }
            Operator::Like => {
                if Self::operator_like(&value_left, &value_right)? {
                    Ok(true)
                } else {
                    Ok(false)
                }
            }
            Operator::NotLike => {
                if !Self::operator_like(&value_left, &value_right)? {
                    Ok(true)
                } else {
                    Ok(false)
                }
            }
            Operator::In => {
                if Self::operator_in(&value_left, &value_right)? {
                    Ok(true)
                } else {
                    Ok(false)
                }
            }
            Operator::NotIn => {
                if !Self::operator_in(&value_left, &value_right)? {
                    Ok(true)
                } else {
                    Ok(false)
                }
            }
            Operator::Between => {
                if Self::operator_between(&value_left, &value_right)? {
                    Ok(true)
                } else {
                    Ok(false)
                }
            }
            Operator::NotBetween => {
                if !Self::operator_between(&value_left, &value_right)? {
                    Ok(true)
                } else {
                    Ok(false)
                }
            }
            Operator::IsNull => {
                if value_left.is_null() {
                    Ok(true)
                } else {
                    Ok(false)
                }
            }
            Operator::IsNotNull => {
                if !value_left.is_null() {
                    Ok(true)
                } else {
                    Ok(false)
                }
            }
            Operator::Regex => {
                if Self::operator_regex(&value_left, &value_right)? {
                    Ok(true)
                } else {
                    Ok(false)
                }
            }
            Operator::NotRegex => {
                if !Self::operator_regex(&value_left, &value_right)? {
                    Ok(true)
                } else {
                    Ok(false)
                }
            }
            _ => Ok(false),
        }
    }

    pub fn resolve_condition_variable(variable: &Value, value: &Value) -> Result<Value, Error> {
        if variable.is_string() {
            match Self::extract_sql_string(&variable.as_string()) {
                Some(val) => Ok(Value::from(val)),
                None => {
                    let variable_str = variable.as_str();

                    match value.get(variable_str) {
                        Some(val) => Ok(val.clone()),
                        None => match Value::try_from(variable_str) {
                            Ok(val) => Ok(val),
                            Err(_) => Err(Error::ConditionVariableNotFound),
                        },
                    }
                }
            }
        } else {
            Ok(variable.clone())
        }
    }

    // if value is beteween ' ' ou " " then return content else return None
    pub fn extract_sql_string(value: &String) -> Option<String> {
        let mut chars = value.chars();
        let first_char = chars.next();
        let last_char = chars.next_back();

        if first_char == last_char && (first_char == Some('\'') || first_char == Some('"')) {
            let mut result = String::new();
            for c in value.chars().skip(1).take(value.len() - 2) {
                result.push(c);
            }
            Some(result)
        } else {
            None
        }
    }

    pub fn operator_like(value_left: &Value, value_right: &Value) -> Result<bool, Error> {
        let left = match value_left.as_string_b() {
            Some(val) => val.as_string(),
            None => return Err(Error::LeftConditionNotString),
        };
        let mut right = match value_right.as_string_b() {
            Some(val) => val.as_string(),
            None => return Err(Error::RightConditionNotString),
        };

        // no use regex
        if right.starts_with('%') && right.ends_with('%') {
            right = right.trim_matches('%').to_string();
            if left.contains(&right) {
                return Ok(true);
            }
        } else if right.starts_with('%') {
            right = right.trim_start_matches('%').to_string();
            if left.ends_with(&right) {
                return Ok(true);
            }
        } else if right.ends_with('%') {
            right = right.trim_end_matches('%').to_string();
            if left.starts_with(&right) {
                return Ok(true);
            }
        } else {
            if left.eq(&right) {
                return Ok(true);
            }
        }

        Ok(false)
    }

    fn operator_regex(value_left: &Value, value_right: &Value) -> Result<bool, Error> {
        let left = match value_left.as_string_b() {
            Some(val) => val.as_string(),
            None => return Err(Error::LeftConditionNotString),
        };
        let right = match value_right.as_string_b() {
            Some(val) => val.as_string(),
            None => return Err(Error::RightConditionNotString),
        };

        let re = Regex::new(&right).unwrap();
        if re.is_match(&left) {
            return Ok(true);
        }

        Ok(false)
    }

    pub fn operator_in(value_left: &Value, value_right: &Value) -> Result<bool, Error> {
        let left = match value_left.as_string_b() {
            Some(val) => val.as_string(),
            None => return Err(Error::LeftConditionNotString),
        };
        let right = match value_right.as_array() {
            Some(val) => val,
            None => return Err(Error::RightConditionNotString),
        };

        for value in right {
            let value = match value.as_string_b() {
                Some(val) => val.as_string(),
                None => return Err(Error::RightConditionNotString),
            };
            if left.eq(&value) {
                return Ok(true);
            }
        }

        Ok(false)
    }

    pub fn operator_between(value_left: &Value, value_right: &Value) -> Result<bool, Error> {
        if !value_right.is_array() && value_right.len() != 2 {
            return Err(Error::BetweenConditionInvalid);
        }

        let value1 = match value_right.get(0) {
            Some(val) => val,
            None => return Err(Error::BetweenConditionInvalid),
        };

        let value2 = match value_right.get(1) {
            Some(val) => val,
            None => return Err(Error::BetweenConditionInvalid),
        };

        if value_left.ge(&value1) && value_left.le(&value2) {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

#[macro_export]
macro_rules! sql_string {
    ($string:expr) => {
        format!("'{}'", $string)
    };
}

#[cfg(test)]
mod tests {
    use super::{Clause, Condition, ConditionGroup, ConditionToken, LogicalOperator, Operator};
    use valu3::prelude::*;

    // Clause: name = 'John'
    #[test]
    fn test_condition_equal() {
        let clause = Clause::condition(Operator::Equal, "name", sql_string!("John"));

        let value = Value::from(vec![("name", "John")]);
        let result = match clause.execute(&value) {
            Ok(result) => result,
            Err(err) => panic!("{}", err),
        };

        assert!(result)
    }

    // Clause: name <> 'John'
    #[test]
    fn test_condition_not_equal() {
        let clause = Clause::condition(Operator::NotEqual, "name", sql_string!("John"));

        let value = Value::from(vec![("name", "John")]);
        let result = match clause.execute(&value) {
            Ok(result) => result,
            Err(err) => panic!("{}", err),
        };

        assert!(!result);
    }

    // Clause: age < 18
    #[test]
    fn test_condition_greater_than() {
        let clause = Clause::condition(Operator::GreaterThan, "age", 18);

        let value = Value::from(vec![("age", 19)]);
        let result = match clause.execute(&value) {
            Ok(result) => result,
            Err(err) => panic!("{}", err),
        };

        assert!(result);
    }

    // Clause: age >= 18
    #[test]
    fn test_condition_greater_than_or_equal() {
        let clause = Clause::condition(Operator::GreaterThanOrEqual, "age", 18);

        let value = Value::from(vec![("age", 18)]);
        let result = match clause.execute(&value) {
            Ok(result) => result,
            Err(err) => panic!("{}", err),
        };

        assert!(result);
    }

    // Clause: age < 18
    #[test]
    fn test_condition_less_than() {
        let clause = Clause::condition(Operator::LessThan, "age", 18);

        let value = Value::from(vec![("age", 17)]);
        let result = match clause.execute(&value) {
            Ok(result) => result,
            Err(err) => panic!("{}", err),
        };

        assert!(result);
    }

    // Clause: age <= 18
    #[test]
    fn test_condition_less_than_or_equal() {
        let clause = Clause::condition(Operator::LessThanOrEqual, "age", 18);

        let value = Value::from(vec![("age", 18)]);
        let result = match clause.execute(&value) {
            Ok(result) => result,
            Err(err) => panic!("{}", err),
        };

        assert!(result);
    }

    // Clause: name LIKE 'J%'
    #[test]
    fn test_condition_like() {
        let clause = Clause::condition(Operator::Like, "name", sql_string!("J%"));

        let value = Value::from(vec![("name", "John")]);
        let result = match clause.execute(&value) {
            Ok(result) => result,
            Err(err) => panic!("{}", err),
        };

        assert!(result);
    }

    // Clause: name NOT LIKE 'J%'
    #[test]
    fn test_condition_not_like() {
        let clause = Clause::condition(Operator::NotLike, "name", sql_string!("J%"));

        let value = Value::from(vec![("name", "John")]);
        let result = match clause.execute(&value) {
            Ok(result) => result,
            Err(err) => panic!("{}", err),
        };

        assert!(!result);
    }

    // Clause: name IN ('John', 'Jane')
    #[test]
    fn test_condition_in() {
        let clause = Clause::condition(Operator::In, "name", Value::from(vec!["John", "Jane"]));

        let value = Value::from(vec![("name", "John")]);
        let result = match clause.execute(&value) {
            Ok(result) => result,
            Err(err) => panic!("{}", err),
        };

        assert!(result);
    }

    // Clause: name NOT IN ('John', 'Jane')
    #[test]
    fn test_condition_not_in() {
        let clause = Clause::condition(Operator::NotIn, "name", Value::from(vec!["John", "Jane"]));

        let value = Value::from(vec![("name", "John")]);
        let result = match clause.execute(&value) {
            Ok(result) => result,
            Err(err) => panic!("{}", err),
        };

        assert!(!result);
    }

    // Clause: age BETWEEN 18 and 20
    #[test]
    fn test_condition_between() {
        let clause = Clause::condition(Operator::Between, "age", Value::from(vec![18, 20]));

        let value = Value::from(vec![("age", 19)]);
        let result = match clause.execute(&value) {
            Ok(result) => result,
            Err(err) => panic!("{}", err),
        };

        assert!(result);
    }

    // Clause: age NOT BETWEEN 18 and 20
    #[test]
    fn test_condition_not_between() {
        let clause = Clause::condition(Operator::NotBetween, "age", Value::from(vec![18, 20]));

        let value = Value::from(vec![("age", 19)]);
        let result = match clause.execute(&value) {
            Ok(result) => result,
            Err(err) => panic!("{}", err),
        };

        assert!(!result);
    }

    // Clause: age IS NULL
    #[test]
    fn test_condition_is_null() {
        let clause = Clause::condition(Operator::IsNull, "age", Value::from(vec![18, 20]));

        let value = Value::from(vec![("age", 19)]);
        let result = match clause.execute(&value) {
            Ok(result) => result,
            Err(err) => panic!("{}", err),
        };

        assert!(!result);
    }

    // Clause: age IS NOT NULL
    #[test]
    fn test_condition_is_not_null() {
        let clause = Clause::condition(Operator::IsNotNull, "age", Value::from(vec![18, 20]));

        let value = Value::from(vec![("age", 19)]);
        let result = match clause.execute(&value) {
            Ok(result) => result,
            Err(err) => panic!("{}", err),
        };

        assert!(result);
    }

    // Clause: name = 'J.*'
    #[test]
    fn test_condition_regex() {
        let clause = Clause::condition(Operator::Regex, "name", sql_string!("J.*"));

        let value = Value::from(vec![("name", "John")]);
        let result = match clause.execute(&value) {
            Ok(result) => result,
            Err(err) => panic!("{}", err),
        };

        assert!(result);
    }

    // Clause: name != 'J.*'
    #[test]
    fn test_condition_not_regex() {
        let clause = Clause::condition(Operator::NotRegex, "name", sql_string!("J.*"));

        let value = Value::from(vec![("name", "John")]);
        let result = match clause.execute(&value) {
            Ok(result) => result,
            Err(err) => panic!("{}", err),
        };

        assert!(!result);
    }

    // Clause: ((name = 'John' AND age = 18) OR name NOT REGEX 'A.*') AND birth_date BETWEEN '1980-01-01' AND '1990-01-01'
    #[test]
    fn test_condition_complex() {
        let clause = Clause::ConditionGroup(ConditionGroup {
            conditions: vec![
                ConditionToken::ConditionGroup(ConditionGroup {
                    conditions: vec![
                        ConditionToken::ConditionGroup(ConditionGroup {
                            conditions: vec![
                                ConditionToken::Condition(Condition {
                                    operator: Operator::Equal,
                                    left: "name".to_value(),
                                    right: "John".to_value(),
                                }),
                                ConditionToken::LogicalOperator(LogicalOperator::And),
                                ConditionToken::Condition(Condition {
                                    operator: Operator::Equal,
                                    left: "age".to_value(),
                                    right: 18.to_value(),
                                }),
                            ],
                        }),
                        ConditionToken::LogicalOperator(LogicalOperator::Or),
                        ConditionToken::Condition(Condition {
                            operator: Operator::NotRegex,
                            left: "name".to_value(),
                            right: "A.*".to_value(),
                        }),
                    ],
                }),
                ConditionToken::LogicalOperator(LogicalOperator::And),
                ConditionToken::Condition(Condition {
                    operator: Operator::Between,
                    left: "birth_date".to_value(),
                    right: vec!["1980-01-01".to_value(), "1990-01-01".to_value()].to_value(),
                }),
            ],
        });

        let value = Value::from(vec![
            ("name", "John".to_value()),
            ("birth_date", "1995-01-01".to_value()),
        ]);

        let result = match clause.execute(&value) {
            Ok(result) => result,
            Err(err) => panic!("{}", err),
        };

        assert!(!result); // birth_date is not between 1980-01-01 and 1990-01-01

        let value = Value::from(vec![
            ("name", "Arial".to_value()),
            ("age", 20.to_value()),
            ("birth_date", "1985-01-01".to_value()),
        ]);

        let result = match clause.execute(&value) {
            Ok(result) => result,
            Err(err) => panic!("{}", err),
        };

        assert!(!result); // name is not John and age is not 18

        let value = Value::from(vec![
            ("name", "John".to_value()),
            ("age", 18.to_value()),
            ("birth_date", "1985-01-01".to_value()),
        ]);

        let result = match clause.execute(&value) {
            Ok(result) => result,
            Err(err) => panic!("{}", err),
        };

        assert!(result); // name is not John and age is 18 and birth_date is between 1980-01-01 and 1990-01-01

        let value = Value::from(vec![
            ("name", "Arial".to_value()),
            ("age", 18.to_value()),
            ("birth_date", "1985-01-01".to_value()),
        ]);

        let result = match clause.execute(&value) {
            Ok(result) => result,
            Err(err) => panic!("{}", err),
        };

        assert!(!result); // name is not John and name is start with A.*
    } 
}
