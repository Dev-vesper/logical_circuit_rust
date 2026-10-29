//! NOR: complement of OR.

use super::{GateImpl, Value};

pub struct Nor;

impl GateImpl for Nor {
    const ARITY: usize = 2;
    const NAME: &'static str = "NOR";

    fn eval3(inputs: &[Value]) -> Value {
        if inputs.iter().any(|&v| v == Value::High) {
            Value::Low
        } else if inputs.iter().all(|&v| v == Value::Low) {
            Value::High
        } else {
            Value::Unknown
        }
    }
}
