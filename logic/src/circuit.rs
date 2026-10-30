//! A combinational circuit: a directed acyclic graph of inputs and gates.

use crate::gates::{Gate, Value};

pub type NodeId = usize;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Input,
    Gate(Gate),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    UnknownNode(NodeId),
    NotAGate(NodeId),
    NotAnInput(NodeId),
    ArityExceeded(NodeId),
    Incomplete(NodeId),
    Cyclic(Vec<NodeId>),
}

fn close_cycle(ids: &[NodeId]) -> Vec<NodeId> {
    if ids.is_empty() {
        return Vec::new();
    }
    let mut v: Vec<NodeId> = ids.to_vec();
    v.push(ids[0]);
    v
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::UnknownNode(id) => write!(f, "unknown node {}", id),
            Error::NotAGate(id) => {
                write!(f, "node {} is an input node and accepts no wire", id)
            }
            Error::NotAnInput(id) => write!(f, "node {} is a gate, not an input", id),
            Error::ArityExceeded(id) => {
                write!(f, "node {} has every input port wired already", id)
            }
            Error::Incomplete(id) => write!(f, "node {} has unwired input ports", id),
            Error::Cyclic(ids) => {
                let path: Vec<String> = close_cycle(ids).iter().map(|id| id.to_string()).collect();
                write!(f, "feedback cycle: {}", path.join(" -> "))
            }
        }
    }
}

#[derive(Debug, Clone)]
struct Node {
    kind: Kind,
    input: bool,
    wires: Vec<NodeId>,
    successors: Vec<NodeId>,
}

#[derive(Debug, Clone, Default)]
pub struct Circuit {
    nodes: Vec<Node>,
}

impl Circuit {
    // ----- construction and basic inspection -----

    pub fn new() -> Self {
        Circuit { nodes: Vec::new() }
    }

    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    fn find(&self, id: NodeId) -> Result<&Node, Error> {
        self.nodes.get(id).ok_or(Error::UnknownNode(id))
    }

    pub fn kind(&self, id: NodeId) -> Result<Kind, Error> {
        self.find(id).map(|n| n.kind)
    }

    pub fn push(&mut self, kind: Kind) -> NodeId {
        let id = self.nodes.len();
        self.nodes.push(Node {
            kind,
            input: false,
            wires: Vec::new(),
            successors: Vec::new(),
        });
        id
    }

    pub fn add_input(&mut self, value: bool) -> NodeId {
        let id = self.push(Kind::Input);
        self.nodes[id].input = value;
        id
    }

    pub fn add_gate(&mut self, gate: Gate) -> NodeId {
        self.push(Kind::Gate(gate))
    }

    pub fn input_nodes(&self) -> Vec<NodeId> {
        self.nodes
            .iter()
            .enumerate()
            .filter_map(|(id, n)| matches!(n.kind, Kind::Input).then_some(id))
            .collect()
    }

    // ----- edges: read -----

    /// Sources feeding `id`'s input ports, in insertion order.  For an input
    /// node, always empty.
    pub fn wires(&self, id: NodeId) -> Result<Vec<NodeId>, Error> {
        self.find(id).map(|n| n.wires.clone())
    }

    /// Targets that `id`'s output drives, in insertion order.
    pub fn successors(&self, id: NodeId) -> Result<Vec<NodeId>, Error> {
        self.find(id).map(|n| n.successors.clone())
    }

    /// Visit every (src, dst) edge once.  Order: by src index, then by dst
    /// insertion order.
    pub fn iter_wires(&self, mut f: impl FnMut(NodeId, NodeId)) {
        for (src, n) in self.nodes.iter().enumerate() {
            for &dst in &n.successors {
                f(src, dst);
            }
        }
    }

    // ----- edges: mutate -----

    pub fn connect(&mut self, src: NodeId, dst: NodeId) -> Result<(), Error> {
        let src_ok = src < self.nodes.len();
        let dst_ok = dst < self.nodes.len();
        if !src_ok {
            return Err(Error::UnknownNode(src));
        }
        if !dst_ok {
            return Err(Error::UnknownNode(dst));
        }
        match self.nodes[dst].kind {
            Kind::Input => Err(Error::NotAGate(dst)),
            Kind::Gate(g) => {
                if self.nodes[dst].wires.len() >= g.arity() {
                    Err(Error::ArityExceeded(dst))
                } else {
                    self.nodes[dst].wires.push(src);
                    self.nodes[src].successors.push(dst);
                    Ok(())
                }
            }
        }
    }

    pub fn connect_exn(&mut self, src: NodeId, dst: NodeId) {
        self.connect(src, dst)
            .unwrap_or_else(|e| panic!("{}", e));
    }

    // ----- input values -----

    pub fn set_input(&mut self, id: NodeId, value: bool) -> Result<(), Error> {
        match self.find(id)? {
            Node { kind: Kind::Input, .. } => {
                self.nodes[id].input = value;
                Ok(())
            }
            _ => Err(Error::NotAnInput(id)),
        }
    }

    /// Current value of an input node, or `None` if `id` is unknown or not
    /// an input.  The symmetric reader for [`Circuit::set_input`].
    pub fn get_input(&self, id: NodeId) -> Option<bool> {
        self.nodes.get(id).and_then(|n| {
            matches!(n.kind, Kind::Input).then_some(n.input)
        })
    }

    // ----- simulation -----

    /// Port wire list padded to the gate's arity with [None].
    fn ports(g: Gate, wires: &[NodeId]) -> Vec<Option<NodeId>> {
        (0..g.arity()).map(|i| wires.get(i).copied()).collect()
    }

    pub fn simulate_partial(&self) -> (Vec<Value>, Vec<NodeId>) {
        #[derive(Clone, Copy, PartialEq, Eq)]
        enum Color {
            White,
            Grey,
            Black,
        }

        let count = self.nodes.len();
        let mut values = vec![Value::Unknown; count];
        let mut colors = vec![Color::White; count];
        let mut path: Vec<NodeId> = Vec::new();
        let mut cycle: Option<Vec<NodeId>> = None;

        fn visit(
            circuit: &Circuit,
            id: NodeId,
            values: &mut [Value],
            colors: &mut [Color],
            path: &mut Vec<NodeId>,
            cycle: &mut Option<Vec<NodeId>>,
        ) {
            match colors[id] {
                Color::Black => {}
                Color::Grey => {
                    if cycle.is_none() {
                        let pos = path.iter().position(|&x| x == id).unwrap_or(0);
                        let seg: Vec<NodeId> = path[pos..].iter().rev().copied().collect();
                        *cycle = Some(seg);
                    }
                }
                Color::White => {
                    colors[id] = Color::Grey;
                    path.push(id);
                    let wires = circuit.nodes[id].wires.clone();
                    for w in wires {
                        visit(circuit, w, values, colors, path, cycle);
                    }
                    values[id] = match circuit.nodes[id].kind {
                        Kind::Input => Value::of_bool(circuit.nodes[id].input),
                        Kind::Gate(g) => {
                            let inputs: Vec<Value> =
                                Circuit::ports(g, &circuit.nodes[id].wires)
                                    .into_iter()
                                    .map(|opt| match opt {
                                        Some(w) => values[w],
                                        None => Value::Unknown,
                                    })
                                    .collect();
                            g.eval3(&inputs)
                        }
                    };
                    colors[id] = Color::Black;
                    path.pop();
                }
            }
        }

        for id in 0..count {
            visit(self, id, &mut values, &mut colors, &mut path, &mut cycle);
        }
        (values, cycle.unwrap_or_default())
    }

    fn first_unwired(&self) -> Option<NodeId> {
        for (id, n) in self.nodes.iter().enumerate() {
            if let Kind::Gate(g) = n.kind {
                if n.wires.len() < g.arity() {
                    return Some(id);
                }
            }
        }
        None
    }

    pub fn simulate(&self) -> Result<Vec<bool>, Error> {
        let (values, cycle) = self.simulate_partial();
        if !cycle.is_empty() {
            return Err(Error::Cyclic(cycle));
        }
        if let Some(id) = self.first_unwired() {
            return Err(Error::Incomplete(id));
        }
        Ok(values.into_iter().map(|v| v.to_bool()).collect())
    }

    /// Boolean value of a single node.  Fully evaluates the circuit first;
    /// use [`Circuit::simulate_partial`] if partial values are acceptable.
    pub fn eval(&self, id: NodeId) -> Result<bool, Error> {
        if id >= self.nodes.len() {
            return Err(Error::UnknownNode(id));
        }
        let values = self.simulate()?;
        Ok(values[id])
    }

    /// Full simulation, discarding the values.  Equivalent to
    /// `self.simulate().map(|_| ())`.
    pub fn validate(&self) -> Result<(), Error> {
        self.simulate().map(|_| ())
    }
}
