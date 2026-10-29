//! Exclusive OR: high when an odd number of inputs are high.

use super::{GateImpl, Value};

pub struct Xor;

impl GateImpl for Xor {
    const ARITY: usize = 2;
    const NAME: &'static str = "XOR";

    fn eval3(inputs: &[Value]) -> Value {
        let mut parity = false;
        for &v in inputs {
            match v {
                Value::High => parity = !parity,
                Value::Low => {}
                Value::Unknown => return Value::Unknown,
            }
        }
        Value::of_bool(parity)
    }
}
