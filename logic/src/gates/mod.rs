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

#[cfg(test)]
mod tests {
    use super::*;

    fn t(g: Gate, a: Value, b: Value) -> Value {
        g.eval3(&[a, b])
    }

    #[test]
    fn xor_truth_table() {
        use Value::*;
        assert_eq!(t(Gate::Xor, Low, Low), Low);
        assert_eq!(t(Gate::Xor, Low, High), High);
        assert_eq!(t(Gate::Xor, High, Low), High);
        assert_eq!(t(Gate::Xor, High, High), Low);
        assert_eq!(t(Gate::Xor, Unknown, High), Unknown);
        assert_eq!(t(Gate::Xor, Low, Unknown), Unknown);
    }

    #[test]
    fn xnor_is_not_xor() {
        use Value::*;
        for &a in &[Low, High, Unknown] {
            for &b in &[Low, High, Unknown] {
                let x = Gate::Xor.eval3(&[a, b]);
                let want = match x {
                    Low => High,
                    High => Low,
                    Unknown => Unknown,
                };
                assert_eq!(Gate::Xnor.eval3(&[a, b]), want);
            }
        }
    }

    #[test]
    fn nand_is_not_and() {
        use Value::*;
        for &a in &[Low, High, Unknown] {
            for &b in &[Low, High, Unknown] {
                let x = Gate::And.eval3(&[a, b]);
                let want = match x {
                    Low => High,
                    High => Low,
                    Unknown => Unknown,
                };
                assert_eq!(Gate::Nand.eval3(&[a, b]), want);
            }
        }
    }

    #[test]
    fn nor_is_not_or() {
        use Value::*;
        for &a in &[Low, High, Unknown] {
            for &b in &[Low, High, Unknown] {
                let x = Gate::Or.eval3(&[a, b]);
                let want = match x {
                    Low => High,
                    High => Low,
                    Unknown => Unknown,
                };
                assert_eq!(Gate::Nor.eval3(&[a, b]), want);
            }
        }
    }
}
