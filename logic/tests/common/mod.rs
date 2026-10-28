use logic::{Circuit, Gate, NodeId};

pub struct HalfAdder {
    pub circuit: Circuit,
    pub a: NodeId,
    pub b: NodeId,
    pub sum: NodeId,
    pub carry: NodeId,
}

pub fn half_adder() -> HalfAdder {
    let mut circuit = Circuit::new();
    let a = circuit.add_input(false);
    let b = circuit.add_input(false);
    let not_a = circuit.add_gate(Gate::Not);
    let not_b = circuit.add_gate(Gate::Not);
    let a_and_not_b = circuit.add_gate(Gate::And);
    let not_a_and_b = circuit.add_gate(Gate::And);
    let sum = circuit.add_gate(Gate::Or);
    let carry = circuit.add_gate(Gate::And);
    for (src, dst) in [
        (a, not_a),
        (b, not_b),
        (a, a_and_not_b),
        (not_b, a_and_not_b),
        (not_a, not_a_and_b),
        (b, not_a_and_b),
        (a_and_not_b, sum),
        (not_a_and_b, sum),
        (a, carry),
        (b, carry),
    ] {
        circuit.connect_exn(src, dst);
    }
    HalfAdder {
        circuit,
        a,
        b,
        sum,
        carry,
    }
}
