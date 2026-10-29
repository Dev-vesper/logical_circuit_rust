//! The OR gate: high if any input is high, low if all are low.

use super::{GateImpl, Value};

pub struct Or;

impl GateImpl for Or {
    const ARITY: usize = 2;
    const NAME: &'static str = "OR";

    fn eval3(inputs: &[Value]) -> Value {
        if inputs.iter().any(|&v| v == Value::High) {
            Value::High
        } else if inputs.iter().all(|&v| v == Value::Low) {
            Value::Low
        } else {
            Value::Unknown
        }
    }
}
