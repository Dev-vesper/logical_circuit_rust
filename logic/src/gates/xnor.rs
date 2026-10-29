//! XNOR: complement of XOR (the "equivalence" gate).

use super::{GateImpl, Value};

pub struct Xnor;

impl GateImpl for Xnor {
    const ARITY: usize = 2;
    const NAME: &'static str = "XNOR";

    fn eval3(inputs: &[Value]) -> Value {
        let mut parity = false;
        for &v in inputs {
            match v {
                Value::High => parity = !parity,
                Value::Low => {}
                Value::Unknown => return Value::Unknown,
            }
        }
        Value::of_bool(!parity)
    }
}
