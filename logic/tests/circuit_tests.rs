mod common;

use logic::circuit::{Error, Kind};
use logic::{Circuit, Gate};

fn is_input(k: Result<Kind, Error>) -> bool {
    matches!(k, Ok(Kind::Input))
}
fn is_gate(k: Result<Kind, Error>) -> bool {
    matches!(k, Ok(Kind::Gate(_)))
}

#[test]
fn structure() {
    let mut c = Circuit::new();
    assert_eq!(c.node_count(), 0);
    let a = c.add_input(true);
    let g = c.add_gate(Gate::Not);
    assert_eq!(a, 0);
    assert_eq!(g, 1);
    assert_eq!(c.node_count(), 2);
    assert!(is_input(c.kind(a)));
    assert!(is_gate(c.kind(g)));
    assert_eq!(c.input_nodes(), vec![a]);
    assert_eq!(c.kind(99), Err(Error::UnknownNode(99)));
}

#[test]
fn wiring_and_ports() {
    let mut c = Circuit::new();
    let a = c.add_input(false);
    let b = c.add_input(false);
    let g = c.add_gate(Gate::And);
    assert_eq!(c.connect(a, g), Ok(()));
    assert_eq!(c.connect(b, g), Ok(()));
    assert_eq!(c.wires(g), Ok(vec![a, b]));

    let h = c.add_gate(Gate::Or);
    c.connect_exn(b, h);
    c.connect_exn(a, h);
    assert_eq!(c.wires(h), Ok(vec![b, a]));

    assert!(matches!(c.connect(a, g), Err(Error::ArityExceeded(_))));
    assert_eq!(c.connect(g, a), Err(Error::NotAGate(a)));
    assert_eq!(c.connect(99, g), Err(Error::UnknownNode(99)));

    let mut count = 0;
    c.iter_wires(|_, _| count += 1);
    assert_eq!(count, 4);
}

#[test]
fn fan_out() {
    let mut c = Circuit::new();
    let a = c.add_input(true);
    let b = c.add_input(true);
    let inv = c.add_gate(Gate::Not);
    let both = c.add_gate(Gate::And);
    c.connect_exn(a, inv);
    c.connect_exn(a, both);
    c.connect_exn(b, both);
    let mut succ = c.successors(a).unwrap();
    succ.sort();
    let mut want = vec![inv, both];
    want.sort();
    assert_eq!(succ, want);
    assert!(!c.eval(inv).unwrap());
    assert!(c.eval(both).unwrap());
}

#[test]
fn input_values() {
    let mut c = Circuit::new();
    let a = c.add_input(false);
    let g = c.add_gate(Gate::Not);
    c.connect_exn(a, g);
    assert_eq!(c.get_input(a), Some(false));
    assert_eq!(c.set_input(a, true), Ok(()));
    assert_eq!(c.get_input(a), Some(true));
    assert_eq!(c.set_input(g, true), Err(Error::NotAnInput(g)));
    assert_eq!(c.get_input(g), None);
    assert_eq!(c.get_input(99), None);
}

#[test]
fn incomplete() {
    let mut c = Circuit::new();
    let dangling = c.add_gate(Gate::Not);
    assert_eq!(c.simulate(), Err(Error::Incomplete(dangling)));
    assert_eq!(c.validate(), Err(Error::Incomplete(dangling)));
    assert!(matches!(c.eval(dangling), Err(Error::Incomplete(_))));
}

#[test]
fn cycles() {
    let mut self_loop = Circuit::new();
    let g = self_loop.add_gate(Gate::Not);
    self_loop.connect_exn(g, g);
    assert_eq!(self_loop.simulate(), Err(Error::Cyclic(vec![g])));

    let mut two = Circuit::new();
    let inv = two.add_gate(Gate::Not);
    let both = two.add_gate(Gate::And);
    let seed = two.add_input(true);
    two.connect_exn(seed, both);
    two.connect_exn(inv, both);
    two.connect_exn(both, inv);
    assert_eq!(two.simulate(), Err(Error::Cyclic(vec![both, inv])));
    assert!(matches!(two.validate(), Err(Error::Cyclic(_))));
    assert!(matches!(two.eval(inv), Err(Error::Cyclic(_))));
    assert!(matches!(two.eval(42), Err(Error::UnknownNode(42))));
    assert!(matches!(two.wires(42), Err(Error::UnknownNode(42))));
}

#[test]
fn half_adder_matches_definition() {
    use common::half_adder;

    let ha = half_adder();
    assert_eq!(ha.circuit.validate(), Ok(()));
    for (va, vb) in [(false, false), (false, true), (true, false), (true, true)] {
        let mut c = ha.circuit.clone();
        c.set_input(ha.a, va).unwrap();
        c.set_input(ha.b, vb).unwrap();
        assert_eq!(c.eval(ha.sum).unwrap(), va ^ vb, "sum a={va} b={vb}");
        assert_eq!(c.eval(ha.carry).unwrap(), va && vb, "carry a={va} b={vb}");
    }
}
