//! The editable board: nodes with positions and input levels, and wires between ports.

use logic::{Circuit, Kind, NodeId, Value};

pub type Id = usize;

#[derive(Debug, Clone)]
pub struct Node {
    pub id: Id,
    pub kind: Kind,
    pub x: f64,
    pub y: f64,
    pub level: bool,
    pub ports: Vec<Option<Id>>,
    pub routes: Vec<Vec<(f64, f64)>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Wire {
    pub src: Id,
    pub dst: Id,
    pub port: usize,
}

#[derive(Debug, Clone, Default)]
pub struct Doc {
    nodes: Vec<Node>,
}

fn arity(kind: &Kind) -> usize {
    match kind {
        Kind::Input => 0,
        Kind::Gate(g) => g.arity(),
    }
}

impl Doc {
    pub fn new() -> Self {
        Doc { nodes: Vec::new() }
    }

    pub fn count(&self) -> usize {
        self.nodes.len()
    }

    pub fn node(&self, id: Id) -> &Node {
        &self.nodes[id]
    }

    pub fn iter(&self, mut f: impl FnMut(&Node)) {
        for n in &self.nodes {
            f(n);
        }
    }

    pub fn wires(&self) -> Vec<Wire> {
        let mut found = Vec::new();
        for n in &self.nodes {
            for (port, p) in n.ports.iter().enumerate() {
                if let Some(src) = p {
                    found.push(Wire {
                        src: *src,
                        dst: n.id,
                        port,
                    });
                }
            }
        }
        found
    }

    pub fn add(&mut self, kind: Kind, x: f64, y: f64) -> Id {
        let id = self.nodes.len();
        let n = arity(&kind);
        self.nodes.push(Node {
            id,
            kind,
            x,
            y,
            level: false,
            ports: vec![None; n],
            routes: vec![Vec::new(); n],
        });
        id
    }

    pub fn move_to(&mut self, id: Id, x: f64, y: f64) {
        self.nodes[id].x = x;
        self.nodes[id].y = y;
    }

    pub fn toggle(&mut self, id: Id) {
        self.nodes[id].level = !self.nodes[id].level;
    }

    pub fn free_port(&self, dst: Id) -> Option<usize> {
        let n = &self.nodes[dst];
        (0..n.ports.len()).find(|&i| n.ports[i].is_none())
    }

    pub fn connect(&mut self, src: Id, dst: Id, port: Option<usize>) -> Option<usize> {
        let n = &mut self.nodes[dst];
        let len = n.ports.len();
        let chosen = match port {
            Some(p) if p < len && n.ports[p].is_none() => Some(p),
            _ => (0..len).find(|&i| n.ports[i].is_none()),
        };
        if let Some(p) = chosen {
            n.ports[p] = Some(src);
        }
        chosen
    }

    pub fn disconnect(&mut self, dst: Id, port: usize) {
        let n = &mut self.nodes[dst];
        n.ports[port] = None;
        n.routes[port] = Vec::new();
    }

    pub fn route(&self, dst: Id, port: usize) -> Vec<(f64, f64)> {
        let n = &self.nodes[dst];
        match n.ports[port] {
            None => Vec::new(),
            Some(src) => {
                let s = &self.nodes[src];
                n.routes[port]
                    .iter()
                    .map(|&(ox, oy)| (s.x + ox, s.y + oy))
                    .collect()
            }
        }
    }

    pub fn set_route(&mut self, dst: Id, port: usize, bends: Vec<(f64, f64)>) {
        let src = match self.nodes[dst].ports[port] {
            None => return,
            Some(s) => s,
        };
        let (sx, sy) = (self.nodes[src].x, self.nodes[src].y);
        let offsets: Vec<(f64, f64)> = bends.iter().map(|&(x, y)| (x - sx, y - sy)).collect();
        self.nodes[dst].routes[port] = offsets;
    }

    pub fn remove(&mut self, id: Id) {
        // Clear or shift ports on every remaining node.
        for n in self.nodes.iter_mut() {
            if n.id == id {
                continue;
            }
            for (port_idx, port) in n.ports.iter_mut().enumerate() {
                match port {
                    Some(src) if *src == id => {
                        *port = None;
                        n.routes[port_idx].clear();
                    }
                    Some(src) if *src > id => {
                        *port = Some(*src - 1);
                    }
                    _ => {}
                }
            }
        }
        // Renumber nodes with id > id.
        for n in self.nodes.iter_mut() {
            if n.id > id {
                n.id -= 1;
            }
        }
        self.nodes.remove(id);
    }

    pub fn clear(&mut self) {
        self.nodes.clear();
    }

    pub fn build(&self) -> Circuit {
        let mut c = Circuit::new();
        for n in &self.nodes {
            match n.kind {
                Kind::Input => {
                    c.add_input(n.level);
                }
                Kind::Gate(g) => {
                    c.add_gate(g);
                }
            }
        }
        for n in &self.nodes {
            for p in &n.ports {
                if let Some(src) = p {
                    c.connect_exn(*src, n.id);
                }
            }
        }
        c
    }

    pub fn evaluate(&self) -> (Circuit, Vec<Value>, Vec<NodeId>) {
        let c = self.build();
        let (values, cycle) = c.simulate_partial();
        (c, values, cycle)
    }
}
