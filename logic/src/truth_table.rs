//! Exhaustive evaluation of a circuit over every combination of input values.

use crate::circuit::{Circuit, Error as CircuitError, NodeId, Kind};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Row {
    pub assignment: Vec<bool>,
    pub results: Vec<bool>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Table {
    pub inputs: Vec<NodeId>,
    pub outputs: Vec<NodeId>,
    pub rows: Vec<Row>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    CircuitError(CircuitError),
    TooManyInputs(usize),
}

pub const MAX_INPUTS: usize = 16;

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::CircuitError(e) => write!(f, "{}", e),
            Error::TooManyInputs(n) => write!(
                f,
                "refusing to enumerate {} inputs (the maximum is {})",
                n, MAX_INPUTS
            ),
        }
    }
}

fn default_outputs(circuit: &Circuit) -> Vec<NodeId> {
    let count = circuit.node_count();
    let mut acc = Vec::new();
    for id in 0..count {
        let is_gate = matches!(circuit.kind(id), Ok(Kind::Gate(_)));
        let is_sink = matches!(circuit.successors(id), Ok(s) if s.is_empty());
        if is_gate && is_sink {
            acc.push(id);
        }
    }
    acc
}

fn name_of_index(k: usize) -> String {
    if k < 26 {
        ((b'A' + k as u8) as char).to_string()
    } else {
        format!("in{}", k)
    }
}

impl Table {
    pub const MAX_INPUTS: usize = crate::truth_table::MAX_INPUTS;
    pub fn of_circuit(circuit: &Circuit, outputs: Option<&[NodeId]>) -> Result<Table, Error> {
        let inputs = circuit.input_nodes();
        let count = inputs.len();
        if count > MAX_INPUTS {
            return Err(Error::TooManyInputs(count));
        }

        let mut work = circuit.clone();
        let outputs: Vec<NodeId> = match outputs {
            Some(given) => given.to_vec(),
            None => default_outputs(circuit),
        };

        let total = 1usize << count;
        let mut rows = Vec::with_capacity(total);
        for i in 0..total {
            for (k, &id) in inputs.iter().enumerate() {
                let bit = (i >> (count - 1 - k)) & 1 == 1;
                work.set_input(id, bit).map_err(Error::CircuitError)?;
            }
            let values = work.simulate().map_err(Error::CircuitError)?;
            let assignment: Vec<bool> = (0..count)
                .map(|k| (i >> (count - 1 - k)) & 1 == 1)
                .collect();
            let results: Vec<bool> = outputs.iter().map(|&id| values[id]).collect();
            rows.push(Row { assignment, results });
        }

        Ok(Table {
            inputs,
            outputs,
            rows,
        })
    }

    pub fn to_string_aligned(&self) -> String {
        let bit_string = |b: bool| if b { "1".to_string() } else { "0".to_string() };
        let inputs_header: String = (0..self.inputs.len())
            .map(name_of_index)
            .collect::<Vec<_>>()
            .join(" ");
        let output_names: Vec<String> =
            self.outputs.iter().map(|id| format!("out{}", id)).collect();
        let width = output_names
            .iter()
            .map(|s| s.len())
            .fold(1, usize::max);
        let outputs_block = |cells: &[String]| -> String {
            cells
                .iter()
                .map(|c| format!("{:<width$}", c, width = width))
                .collect::<Vec<_>>()
                .join(" ")
        };
        let join = |left: &str, right: &str| -> String {
            if left.is_empty() {
                right.to_string()
            } else if right.is_empty() {
                left.to_string()
            } else {
                format!("{} | {}", left, right)
            }
        };
        let line = |assignment: &[bool], results: &[bool]| -> String {
            let left = assignment
                .iter()
                .map(|&b| bit_string(b))
                .collect::<Vec<_>>()
                .join(" ");
            let right = outputs_block(
                &results.iter().map(|&b| bit_string(b)).collect::<Vec<_>>(),
            );
            join(&left, &right)
        };

        let header = join(&inputs_header, &outputs_block(&output_names));
        let ruler: String = header
            .chars()
            .map(|c| match c {
                '|' => '+',
                ' ' => ' ',
                _ => '-',
            })
            .collect();
        let mut lines = vec![header, ruler];
        for row in &self.rows {
            lines.push(line(&row.assignment, &row.results));
        }
        lines.join("\n")
    }
}
