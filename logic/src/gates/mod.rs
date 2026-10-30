//! The primary gates, each in its own module, plus the three-valued `Value`.
//!
//! Each gate is a unit struct implementing [`GateImpl`]; the [`Gate`] enum is
//! a thin façade that dispatches to them. Adding a new gate is therefore one
//! new file plus one variant here — nothing else in `logic` needs to change.

pub mod and;
pub mod nand;
pub mod nor;
pub mod not;
pub mod or;
pub mod xnor;
pub mod xor;

pub use and::And;
pub use nand::Nand;
pub use nor::Nor;
pub use not::Not;
pub use or::Or;
pub use xnor::Xnor;
pub use xor::Xor;

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
    Xor,
    Nand,
    Nor,
    Xnor,
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
            Gate::And
            | Gate::Or
            | Gate::Xor
            | Gate::Nand
            | Gate::Nor
            | Gate::Xnor => 2,
            Gate::Not => 1,
        }
    }

    pub fn eval3(self, inputs: &[Value]) -> Value {
        match self {
            Gate::And => And::eval3(inputs),
            Gate::Or => Or::eval3(inputs),
            Gate::Not => Not::eval3(inputs),
            Gate::Xor => Xor::eval3(inputs),
            Gate::Nand => Nand::eval3(inputs),
            Gate::Nor => Nor::eval3(inputs),
            Gate::Xnor => Xnor::eval3(inputs),
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Gate::And => And::NAME,
            Gate::Or => Or::NAME,
            Gate::Not => Not::NAME,
            Gate::Xor => Xor::NAME,
            Gate::Nand => Nand::NAME,
            Gate::Nor => Nor::NAME,
            Gate::Xnor => Xnor::NAME,
        }
    }
}

impl Value {
    pub fn of_bool(b: bool) -> Self {
        if b {
            Value::High
        } else {
            Value::Low
        }
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
