mod common;
use common::half_adder;
use logic::circuit::{Circuit, Error as CircuitError, Kind};
use logic::{Gate, TruthTable};

#[test]
fn half_adder_table() {
    let mut ha = half_adder();
    let table = TruthTable::of_circuit(&ha.circuit, Some(&[ha.sum, ha.carry])).unwrap();
    assert_eq!(table.outputs, vec![ha.sum, ha.carry]);
    assert_eq!(table.rows.len(), 4);
    let expected = [
        ([false, false], [false, false]),
        ([false, true], [true, false]),
        ([true, false], [true, false]),
        ([true, true], [false, true]),
    ];
    for (row, (a, r)) in table.rows.iter().zip(expected) {
        assert_eq!(row.assignment, a);
        assert_eq!(row.results, r);
    }
    assert_eq!(
        table
            .to_string_aligned()
            .split('\n')
            .count(),
        6
    );
    ha.circuit.set_input(ha.a, true).unwrap();
    let _ = ha.circuit.simulate_partial();
}

#[test]
fn default_outputs_are_sinks() {
    let ha = half_adder();
    let table = TruthTable::of_circuit(&ha.circuit, None).unwrap();
    assert_eq!(table.outputs, vec![ha.sum, ha.carry]);
    assert!(table
        .outputs
        .iter()
        .all(|&id| id != ha.a && id != ha.b));
}

#[test]
fn circuit_untouched() {
    // `of_circuit` takes &Circuit and clones, so the original is untouched.
    let mut ha = half_adder();
    ha.circuit.set_input(ha.a, true).unwrap();
    ha.circuit.set_input(ha.b, true).unwrap();
    let table = TruthTable::of_circuit(&ha.circuit, None).unwrap();
    let last = table.rows.last().unwrap();
    assert_eq!(last.assignment, vec![true, true]);
    assert_eq!(last.results, vec![false, true]);
    assert_eq!(ha.circuit.get_input(ha.a), Some(true));
    assert_eq!(ha.circuit.get_input(ha.b), Some(true));
}

#[test]
fn degenerate_shapes() {
    let mut bare = Circuit::new();
    let _ = bare.add_input(false);
    let table = TruthTable::of_circuit(&bare, None).unwrap();
    assert_eq!(table.rows.len(), 2);
    assert_eq!(table.outputs, Vec::<usize>::new());
    assert_eq!(table.rows.last().unwrap().assignment, vec![true]);

    let empty = Circuit::new();
    let table = TruthTable::of_circuit(&empty, None).unwrap();
    assert_eq!(table.rows.len(), 1);
    assert!(table.rows[0].assignment.is_empty());
    assert_eq!(table.to_string_aligned(), "\n\n");
}

#[test]
fn too_many_inputs() {
    assert_eq!(TruthTable::MAX_INPUTS, 16);
    let mut wide = Circuit::new();
    for _ in 0..=TruthTable::MAX_INPUTS {
        let _ = wide.add_input(false);
    }
    assert_eq!(
        TruthTable::of_circuit(&wide, None),
        Err(logic::TruthTableError::TooManyInputs(17))
    );
}
