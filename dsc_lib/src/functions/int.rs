// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::DscError;
use crate::configure::context::Context;
use crate::functions::AcceptedArgKind;
use serde_json::Value;
use super::Function;

#[derive(Debug, Default)]
pub struct Int {}

impl Function for Int {
    fn accepted_arg_types(&self) -> Vec<AcceptedArgKind> {
        vec![AcceptedArgKind::String, AcceptedArgKind::Number]
    }

    fn min_args(&self) -> usize {
        1
    }

    fn max_args(&self) -> usize {
        1
    }

    fn invoke(&self, args: &[Value], _context: &Context) -> Result<Value, DscError> {
        let arg = &args[0];
        let value: i64;
        if arg.is_string() {
            let input = arg.as_str().ok_or(DscError::Function("int".to_string(), "invalid input string".to_string()))?;
            value = input.parse::<i64>().map_err(|_| DscError::Function("int".to_string(), "unable to parse string to int".to_string()))?;
        } else if arg.is_number() {
            value = arg.as_i64().ok_or(DscError::Function("int".to_string(), "unable to parse number to int".to_string()))?;
        } else {
            return Err(DscError::Function("int".to_string(), "Invalid argument type".to_string()));
        }
        Ok(Value::Number(value.into()))
    }
}

#[cfg(test)]
mod tests {
    use crate::configure::context::Context;
    use crate::parser::Statement;

    #[test]
    fn string() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[int('4')]", &Context::new()).unwrap();
        assert_eq!(result, 4);
    }

    #[test]
    fn number() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[int(123)]", &Context::new()).unwrap();
        assert_eq!(result, 123);
    }

    #[test]
    fn nested() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[int(int('-1'))]", &Context::new()).unwrap();
        assert_eq!(result, -1);
    }

    #[test]
    fn error() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[int('foo')]", &Context::new());
        assert!(result.is_err());
    }
}
