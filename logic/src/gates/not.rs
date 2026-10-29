//! The NOT gate: exactly one input, inverted.

use super::{GateImpl, Value};

pub struct Not;

impl GateImpl for Not {
    const ARITY: usize = 1;
    const NAME: &'static str = "NOT";

    fn eval3(inputs: &[Value]) -> Value {
        match inputs {
            [Value::Low] => Value::High,
            [Value::High] => Value::Low,
            [Value::Unknown] => Value::Unknown,
            _ => panic!("Logic.Gate.eval: Not expects exactly one input"),
        }
    }
}
