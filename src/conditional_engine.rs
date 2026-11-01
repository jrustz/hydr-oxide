use std::str::FromStr;

use reqwest::Response;

#[derive(Debug)]
pub struct Condition {
    key: ConditionalKey,
    op: ConditionalOp,
    value: String,
}

#[derive(Debug)]
pub struct ConditionalEngine {
    pub original_string: String,
    pub conditions: Vec<Condition>,
}

#[derive(Debug, PartialEq)]
enum ConditionalKey {
    CodeStatus,
    ExpectedString,
}

impl FromStr for ConditionalKey {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "C" => Ok(ConditionalKey::CodeStatus),
            "S" => Ok(ConditionalKey::ExpectedString),
            _ => Err(anyhow::anyhow!("Unsupported Conditional key: {}", s)),
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
enum ConditionalOp {
    Eq,
    Gte,
    Gt,
    Lte,
    Lt,
    InEq,
}

impl FromStr for ConditionalOp {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "=" => Ok(ConditionalOp::Eq),
            "!=" => Ok(ConditionalOp::InEq),
            ">=" => Ok(ConditionalOp::Gte),
            ">" => Ok(ConditionalOp::Gt),
            "<=" => Ok(ConditionalOp::Lte),
            "<" => Ok(ConditionalOp::Lt),
            _ => Err(anyhow::anyhow!(
                "Unsupported operation: {}. Supported operation is: =, !=, >=, >, <=, <",
                s
            )),
        }
    }
}

impl ConditionalEngine {
    pub fn new(original_string: &str) -> Self {
        ConditionalEngine {
            original_string: String::from(original_string),
            conditions: Vec::new(),
        }
    }

    //NOTE: string format is: cond_key::cond_op::cond__val;cond_key2::cond_op::cond__val2
    pub fn parse_original_string(mut self) -> Result<Self, anyhow::Error> {
        let input = self.original_string.trim();
        if input.is_empty() {
            return Err(anyhow::anyhow!("Faild to parse epty conditional string"));
        }
        for conditional_string in input.split(';') {
            let mut parts = conditional_string.split("::");
            let key_str = parts
                .next()
                .ok_or_else(|| anyhow::anyhow!("Missing key"))?
                .trim();
            let op_str = parts
                .next()
                .ok_or_else(|| anyhow::anyhow!("Missing operation"))?
                .trim();
            let value_str = parts
                .next()
                .ok_or_else(|| anyhow::anyhow!("Missing value"))?
                .trim();

            let key = ConditionalKey::from_str(key_str)?;
            let op = ConditionalOp::from_str(op_str)?;
            let value = String::from(value_str);
            self.conditions.push(Condition { key, op, value });
        }
        Ok(self)
    }

    //TODO: Now it's AND logic only, add OR
    pub async fn validate(&self, response: Response) -> Result<bool, anyhow::Error> {
        let current_status = response.status().as_u16();

        let current_text = response
            .text()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to read response body: {}", e))?;

        let is_valid = self.conditions.iter().all(|cond| match cond.key {
            ConditionalKey::CodeStatus => {
                if let Ok(target_code) = cond.value.parse::<u16>() {
                    match cond.op {
                        ConditionalOp::Eq => current_status == target_code,
                        ConditionalOp::InEq => current_status != target_code,
                        ConditionalOp::Gte => current_status >= target_code,
                        ConditionalOp::Gt => current_status > target_code,
                        ConditionalOp::Lte => current_status <= target_code,
                        ConditionalOp::Lt => current_status < target_code,
                    }
                } else {
                    false
                }
            }
            ConditionalKey::ExpectedString => match cond.op {
                ConditionalOp::Eq => current_text.contains(&cond.value),
                ConditionalOp::InEq => !current_text.contains(&cond.value),
                _ => false,
            },
        });

        std::result::Result::Ok(is_valid)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_valid_single_condition() {
        let raw = "C::=::200";
        let engine = ConditionalEngine::new(raw).parse_original_string().unwrap();

        assert_eq!(engine.conditions.len(), 1);
        assert_eq!(engine.conditions[0].key, ConditionalKey::CodeStatus);
        assert_eq!(engine.conditions[0].op, ConditionalOp::Eq);
        assert_eq!(engine.conditions[0].value, "200");
    }

    #[test]
    fn test_parse_valid_multyply_conditional() {
        let raw = "C::=::301;S::=::Hello world!";
        let engine = ConditionalEngine::new(raw).parse_original_string().unwrap();
        assert_eq!(engine.conditions.len(), 2);
        assert_eq!(engine.conditions[0].key, ConditionalKey::CodeStatus);
        assert_eq!(engine.conditions[1].key, ConditionalKey::ExpectedString);
        assert_eq!(engine.conditions[0].op, ConditionalOp::Eq);
        assert_eq!(engine.conditions[1].op, ConditionalOp::Eq);
        assert_eq!(engine.conditions[0].value, "301");
        assert_eq!(engine.conditions[1].value, "Hello world!");
    }

    #[test]
    fn test_parse_multiple_conditions_with_spaces() {
        // Тестируем устойчивость к пробелам и разный регистр ключей
        let raw = "  c  :: >= :: 200 ; S :: != :: Error Occurred  ";
        let engine = ConditionalEngine::new(raw).parse_original_string().unwrap();

        assert_eq!(engine.conditions.len(), 2);

        // Проверяем первое условие
        assert_eq!(engine.conditions[0].key, ConditionalKey::CodeStatus);
        assert_eq!(engine.conditions[0].op, ConditionalOp::Gte);
        assert_eq!(engine.conditions[0].value, "200");

        // Проверяем второе условие
        assert_eq!(engine.conditions[1].key, ConditionalKey::ExpectedString);
        assert_eq!(engine.conditions[1].op, ConditionalOp::InEq);
        assert_eq!(engine.conditions[1].value, "Error Occurred");
    }

    #[test]
    fn test_parse_empty_string_fails() {
        let raw = "   ";
        let result = ConditionalEngine::new(raw).parse_original_string();

        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "Faild to parse epty conditional string"
        );
    }

    #[test]
    fn test_parse_invalid_key_fails() {
        let raw = "X::=::100";
        let result = ConditionalEngine::new(raw).parse_original_string();

        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Unsupported Conditional key")
        );
    }

    #[test]
    fn test_parse_invalid_operator_fails() {
        let raw = "C::==::200";
        let result = ConditionalEngine::new(raw).parse_original_string();

        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Unsupported operation")
        );
    }

    #[test]
    fn test_parse_missing_parts_fails() {
        let raw = "C::=";
        let result = ConditionalEngine::new(raw).parse_original_string();

        assert!(result.is_err());
    }
}
