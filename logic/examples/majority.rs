use logic::{Circuit, Gate, NodeId, TruthTable};

fn majority() -> (Circuit, NodeId) {
    let mut circuit = Circuit::new();
    let a = circuit.add_input(false);
    let b = circuit.add_input(false);
    let c = circuit.add_input(false);
    let a_and_b = circuit.add_gate(Gate::And);
    let b_and_c = circuit.add_gate(Gate::And);
    let a_and_c = circuit.add_gate(Gate::And);
    let first_two = circuit.add_gate(Gate::Or);
    let out = circuit.add_gate(Gate::Or);
    for (src, dst) in [
        (a, a_and_b),
        (b, a_and_b),
        (b, b_and_c),
        (c, b_and_c),
        (a, a_and_c),
        (c, a_and_c),
        (a_and_b, first_two),
        (b_and_c, first_two),
        (first_two, out),
        (a_and_c, out),
    ] {
        circuit.connect_exn(src, dst);
    }
    (circuit, out)
}

fn show_majority() {
    let (circuit, out) = majority();
    println!("majority voter: out is 1 when at least two of A, B and C are 1\n");
    match TruthTable::of_circuit(&circuit, Some(&[out])) {
        Ok(table) => {
            print!("{}", table.to_string_aligned());
            println!();
        }
        Err(e) => println!("  failed: {}", e),
    }
}

fn show_rejections() {
    let mut circuit = Circuit::new();
    let a = circuit.add_input(false);
    let b = circuit.add_input(false);
    let conjunction = circuit.add_gate(Gate::And);
    circuit.connect_exn(a, conjunction);
    circuit.connect_exn(b, conjunction);
    println!(
        "a third wire on an And:      {}",
        match circuit.connect(b, conjunction) {
            Ok(()) => "unexpectedly accepted".to_string(),
            Err(e) => e.to_string(),
        }
    );
    println!(
        "a wire aimed at an input:    {}",
        match circuit.connect(conjunction, a) {
            Ok(()) => "unexpectedly accepted".to_string(),
            Err(e) => e.to_string(),
        }
    );

    let mut dangling = Circuit::new();
    let _ = dangling.add_gate(Gate::Not);
    println!(
        "a Not gate with no wire:     {}",
        match dangling.simulate() {
            Ok(_) => "unexpectedly simulated".to_string(),
            Err(e) => e.to_string(),
        }
    );

    let mut feedback = Circuit::new();
    let inverts = feedback.add_gate(Gate::Not);
    feedback.connect_exn(inverts, inverts);
    println!(
        "a Not gate wired to itself:  {}",
        match feedback.simulate() {
            Ok(_) => "unexpectedly simulated".to_string(),
            Err(e) => e.to_string(),
        }
    );
}

fn main() {
    show_majority();
    println!("rejections:");
    show_rejections();
}
