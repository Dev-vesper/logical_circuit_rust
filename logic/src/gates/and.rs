//! The AND gate: low if any input is low, high if all are high.

use super::{GateImpl, Value};

pub struct And;

impl GateImpl for And {
    const ARITY: usize = 2;
    const NAME: &'static str = "AND";

    fn eval3(inputs: &[Value]) -> Value {
        if inputs.iter().any(|&v| v == Value::Low) {
            Value::Low
        } else if inputs.iter().all(|&v| v == Value::High) {
            Value::High
        } else {
            Value::Unknown
        }
    }
}
