//! Integration tests for the gate truth tables, driven exclusively through
//! the crate's public API (`Gate::eval3`, `Value`).

use logic::{Gate, Value};

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
