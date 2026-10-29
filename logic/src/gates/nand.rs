//! NAND: complement of AND.

use super::{GateImpl, Value};

pub struct Nand;

impl GateImpl for Nand {
    const ARITY: usize = 2;
    const NAME: &'static str = "NAND";

    fn eval3(inputs: &[Value]) -> Value {
        if inputs.iter().any(|&v| v == Value::Low) {
            Value::High
        } else if inputs.iter().all(|&v| v == Value::High) {
            Value::Low
        } else {
            Value::Unknown
        }
    }
}
