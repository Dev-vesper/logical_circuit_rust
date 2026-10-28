use logic::{Circuit, Gate, Value};

#[test]
fn short_circuit() {
    let mut and_low = Circuit::new();
    let a = and_low.add_input(false);
    let g = and_low.add_gate(Gate::And);
    and_low.connect_exn(a, g);
    let (values, cycle) = and_low.simulate_partial();
    assert!(cycle.is_empty());
    assert_eq!(values[g], Value::Low);

    let mut or_high = Circuit::new();
    let a = or_high.add_input(true);
    let g = or_high.add_gate(Gate::Or);
    or_high.connect_exn(a, g);
    assert_eq!(or_high.simulate_partial().0[g], Value::High);
}

#[test]
fn undecided_for_want_of_a_wire() {
    let mut c = Circuit::new();
    let a = c.add_input(true);
    let g = c.add_gate(Gate::And);
    c.connect_exn(a, g);
    let dangling = c.add_gate(Gate::Not);
    let h = c.add_gate(Gate::Not);
    c.connect_exn(g, h);
    let (values, _) = c.simulate_partial();
    assert_eq!(values[a], Value::High);
    assert_eq!(values[g], Value::Unknown);
    assert_eq!(values[dangling], Value::Unknown);
    assert_eq!(values[h], Value::Unknown);
}

#[test]
fn rest_keeps_working() {
    let mut c = Circuit::new();
    let a = c.add_input(true);
    let b = c.add_input(true);
    let wired = c.add_gate(Gate::And);
    c.connect_exn(a, wired);
    c.connect_exn(b, wired);
    let _fresh = c.add_gate(Gate::Or);
    let (values, _) = c.simulate_partial();
    assert_eq!(values[wired], Value::High);
}

#[test]
fn cycles_reported() {
    let mut c = Circuit::new();
    let inv = c.add_gate(Gate::Not);
    let both = c.add_gate(Gate::And);
    let seed = c.add_input(true);
    c.connect_exn(seed, both);
    c.connect_exn(inv, both);
    c.connect_exn(both, inv);
    let after = c.add_gate(Gate::Not);
    c.connect_exn(both, after);
    let (values, cycle) = c.simulate_partial();
    assert_eq!(cycle, vec![both, inv]);
    assert_eq!(values[inv], Value::Unknown);
    assert_eq!(values[both], Value::Unknown);
    assert_eq!(values[after], Value::Unknown);
    assert_eq!(values[seed], Value::High);
    assert!(matches!(
        c.simulate(),
        Err(logic::CircuitError::Cyclic(_))
    ));
}

#[test]
fn cycle_wins_over_incomplete() {
    let mut c = Circuit::new();
    let _ = c.add_gate(Gate::And);
    let inv = c.add_gate(Gate::Not);
    c.connect_exn(inv, inv);
    assert_eq!(
        c.simulate(),
        Err(logic::CircuitError::Cyclic(vec![inv]))
    );
    let (_, cycle) = c.simulate_partial();
    assert_eq!(cycle, vec![inv]);
}
