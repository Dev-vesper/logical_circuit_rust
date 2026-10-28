//! The three primary gates, and the single source of truth for gate semantics.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Gate {
    And,
    Or,
    Not,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Value {
    Low,
    High,
    Unknown,
}

impl Gate {
    pub fn arity(self) -> usize {
        match self {
            Gate::And | Gate::Or => 2,
            Gate::Not => 1,
        }
    }

    pub fn eval3(self, inputs: &[Value]) -> Value {
        match self {
            Gate::And => {
                if inputs.iter().any(|&v| v == Value::Low) {
                    Value::Low
                } else if inputs.iter().all(|&v| v == Value::High) {
                    Value::High
                } else {
                    Value::Unknown
                }
            }
            Gate::Or => {
                if inputs.iter().any(|&v| v == Value::High) {
                    Value::High
                } else if inputs.iter().all(|&v| v == Value::Low) {
                    Value::Low
                } else {
                    Value::Unknown
                }
            }
            Gate::Not => match inputs {
                [Value::Low] => Value::High,
                [Value::High] => Value::Low,
                [Value::Unknown] => Value::Unknown,
                _ => panic!("Logic.Gate.eval: Not expects exactly one input"),
            },
        }
    }

    pub fn eval(self, inputs: &[bool]) -> bool {
        let vals: Vec<Value> = inputs.iter().map(|&b| Value::of_bool(b)).collect();
        self.eval3(&vals).to_bool()
    }

    pub fn name(self) -> &'static str {
        match self {
            Gate::And => "AND",
            Gate::Or => "OR",
            Gate::Not => "NOT",
        }
    }
}

impl Value {
    pub fn of_bool(b: bool) -> Self {
        if b { Value::High } else { Value::Low }
    }

    pub fn to_bool(self) -> bool {
        match self {
            Value::High => true,
            Value::Low => false,
            Value::Unknown => panic!("Logic.Gate.to_bool: the value is unknown"),
        }
    }
}

impl std::fmt::Display for Gate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.name())
    }
}
