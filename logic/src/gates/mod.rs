//! The primary gates, each in its own module, plus the three-valued `Value`.
//!
//! Each gate is a unit struct implementing [`GateImpl`]; the [`Gate`] enum is
//! a thin façade that dispatches to them. Adding a new gate is therefore one
//! new file plus one variant here — nothing else in `logic` needs to change.

pub mod and;
pub mod not;
pub mod or;

pub use and::And;
pub use not::Not;
pub use or::Or;

/// The behaviour every gate provides.  Consts keep arity/name discoverable
/// without constructing anything; `eval3` is the single source of truth.
pub trait GateImpl {
    const ARITY: usize;
    const NAME: &'static str;
    fn eval3(inputs: &[Value]) -> Value;
}

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
            Gate::And => And::ARITY,
            Gate::Or  => Or::ARITY,
            Gate::Not => Not::ARITY,
        }
    }

    pub fn eval3(self, inputs: &[Value]) -> Value {
        match self {
            Gate::And => And::eval3(inputs),
            Gate::Or  => Or::eval3(inputs),
            Gate::Not => Not::eval3(inputs),
        }
    }

    pub fn eval(self, inputs: &[bool]) -> bool {
        let vals: Vec<Value> = inputs.iter().map(|&b| Value::of_bool(b)).collect();
        self.eval3(&vals).to_bool()
    }

    pub fn name(self) -> &'static str {
        match self {
            Gate::And => And::NAME,
            Gate::Or  => Or::NAME,
            Gate::Not => Not::NAME,
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
