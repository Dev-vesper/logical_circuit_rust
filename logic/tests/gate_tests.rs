use logic::{Gate, Value};

#[test]
fn arity() {
    assert_eq!(Gate::And.arity(), 2);
    assert_eq!(Gate::Or.arity(), 2);
    assert_eq!(Gate::Not.arity(), 1);
}

#[test]
fn truth_tables() {
    let rows = [
        (false, false),
        (false, true),
        (true, false),
        (true, true),
    ];
    for (a, b) in rows {
        assert_eq!(Gate::And.eval(&[a, b]), a && b, "AND({a},{b})");
        assert_eq!(Gate::Or.eval(&[a, b]), a || b, "OR({a},{b})");
    }
    assert_eq!(Gate::Not.eval(&[false]), true);
    assert_eq!(Gate::Not.eval(&[true]), false);
}

#[test]
fn nary_and_empty() {
    assert!(Gate::And.eval(&[true, true, true]));
    assert!(!Gate::And.eval(&[true, false, true]));
    assert!(Gate::Or.eval(&[false, false, true]));
    assert!(!Gate::Or.eval(&[false, false]));
    assert!(Gate::And.eval(&[]));
    assert!(!Gate::Or.eval(&[]));
}

#[test]
#[should_panic]
fn not_rejects_two_inputs() {
    let _ = Gate::Not.eval(&[true, false]);
}

#[test]
#[should_panic]
fn not_rejects_zero_inputs() {
    let _ = Gate::Not.eval(&[]);
}

#[test]
fn printing() {
    assert_eq!(Gate::And.name(), "AND");
    assert_eq!(Gate::Or.name(), "OR");
    assert_eq!(Gate::Not.name(), "NOT");
}

#[test]
fn eval3_values() {
    assert_eq!(Gate::And.eval3(&[Value::Low, Value::Unknown]), Value::Low);
    assert_eq!(Gate::Or.eval3(&[Value::High, Value::Unknown]), Value::High);
    assert_eq!(
        Gate::And.eval3(&[Value::High, Value::Unknown]),
        Value::Unknown
    );
    assert_eq!(
        Gate::Or.eval3(&[Value::Low, Value::Unknown]),
        Value::Unknown
    );
    assert_eq!(Gate::Not.eval3(&[Value::Low]), Value::High);
    assert_eq!(Gate::Not.eval3(&[Value::High]), Value::Low);
    assert_eq!(Gate::Not.eval3(&[Value::Unknown]), Value::Unknown);
}
