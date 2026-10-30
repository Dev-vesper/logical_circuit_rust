//! The editable board: nodes (components) at grid positions, plus a set of
//! free wire segments. Nodes and wires know nothing about each other; a wire
//! is attached to a pin only because they share a coordinate.

use std::collections::HashSet;

use logic::{Circuit, Kind, NodeId, Value};

use crate::layout;
use crate::wiring::{self, Loc, Nets, Wire, Wires};

pub type Id = usize;

#[derive(Debug, Clone)]
pub struct Node {
    pub id: Id,
    pub kind: Kind,
    pub x: i32,
    pub y: i32,
    pub level: bool,
}

impl Node {
    pub fn inputs(&self) -> Vec<Loc> {
        layout::input_offsets(&self.kind)
            .into_iter()
            .map(|(dx, dy)| (self.x + dx, self.y + dy))
            .collect()
    }

    pub fn output(&self) -> Loc {
        let (dx, dy) = layout::output_offset(&self.kind);
        (self.x + dx, self.y + dy)
    }

    /// Input pins first, the output pin last.
    pub fn pins(&self) -> Vec<Loc> {
        let mut v = self.inputs();
        v.push(self.output());
        v
    }
}

// ---------- analysis ----------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NetState {
    /// No pin on the net: Logisim draws it gray.
    Bare,
    /// Pins but nothing drives the net: blue.
    Floating,
    /// Two outputs on one net: red.
    Conflict,
    Driven(Id),
}

/// What colour a wire / pin at a location should have.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Look {
    Bare,
    Value(Value),
    Conflict,
    Cycle,
}

pub struct Analysis {
    pub circuit: Circuit,
    pub values: Vec<Value>,
    pub cycle: Vec<NodeId>,
    pub nets: Nets,
    pub state: Vec<NetState>,
    pub net_cycle: Vec<bool>,
}

impl Analysis {
    pub fn look(&self, p: Loc) -> Look {
        match self.nets.net_at(p) {
            None => Look::Bare,
            Some(k) => {
                if self.net_cycle[k] {
                    return Look::Cycle;
                }
                match self.state[k] {
                    NetState::Bare => Look::Bare,
                    NetState::Floating => Look::Value(Value::Unknown),
                    NetState::Conflict => Look::Conflict,
                    NetState::Driven(d) => Look::Value(self.values[d]),
                }
            }
        }
    }
}

// ---------- the document ----------

#[derive(Debug, Clone, Default)]
pub struct Doc {
    nodes: Vec<Node>,
    wires: Wires,
}

impl Doc {
    pub fn new() -> Self {
        Doc::default()
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

    pub fn wires(&self) -> &Wires {
        &self.wires
    }

    pub fn pin_locs(&self) -> Vec<Loc> {
        self.nodes.iter().flat_map(|n| n.pins()).collect()
    }

    fn repair(&mut self) {
        let pins: HashSet<Loc> = self.pin_locs().into_iter().collect();
        self.wires.repair(&pins);
    }

    // ----- nodes -----

    pub fn add(&mut self, kind: Kind, x: i32, y: i32) -> Id {
        let id = self.nodes.len();
        self.nodes.push(Node { id, kind, x, y, level: false });
        self.repair(); // a pin dropped onto a wire splits and joins it
        id
    }

    pub fn toggle(&mut self, id: Id) {
        self.nodes[id].level = !self.nodes[id].level;
    }

    pub fn remove(&mut self, id: Id) {
        self.nodes.remove(id);
        for n in self.nodes.iter_mut() {
            if n.id > id {
                n.id -= 1;
            }
        }
        self.repair(); // wires stay, exactly like Logisim
    }

    pub fn clear(&mut self) {
        self.nodes.clear();
        self.wires.clear();
    }

    fn pin_nets(&self, of: &[Loc]) -> Vec<usize> {
        let nets = Nets::build(&self.wires, &self.pin_locs());
        of.iter()
            .map(|p| nets.net_at(*p).unwrap_or(usize::MAX))
            .collect()
    }

    /// Move a node. With `keep` (Logisim's default, Shift turns it off) the
    /// wires hanging on its pins are re-routed so that nothing is
    /// disconnected: the stationary end stays, and the wire is continued by
    /// an L whose first leg runs along the old wire.
    pub fn move_node(&mut self, id: Id, x: i32, y: i32, keep: bool) {
        let (dx, dy) = (x - self.nodes[id].x, y - self.nodes[id].y);
        if dx == 0 && dy == 0 {
            return;
        }
        let old_pins = self.nodes[id].pins();
        let old_set: HashSet<Loc> = old_pins.iter().copied().collect();
        let before = self.pin_nets(&old_pins);
        let snapshot = if keep { Some(self.clone()) } else { None };

        let attached: Vec<Wire> = if keep {
            self.wires
                .iter()
                .filter(|w| old_set.contains(&w.e0) || old_set.contains(&w.e1))
                .copied()
                .collect()
        } else {
            Vec::new()
        };

        self.nodes[id].x = x;
        self.nodes[id].y = y;

        for w in attached {
            self.wires.remove(&w);
            let at0 = old_set.contains(&w.e0);
            let at1 = old_set.contains(&w.e1);
            if at0 && at1 {
                // both ends on this node: the wire travels with it
                self.wires
                    .add((w.e0.0 + dx, w.e0.1 + dy), (w.e1.0 + dx, w.e1.1 + dy));
            } else {
                let (p, q) = if at0 { (w.e0, w.e1) } else { (w.e1, w.e0) };
                let p2 = (p.0 + dx, p.1 + dy);
                let corner = if w.horizontal() { (p2.0, q.1) } else { (q.0, p2.1) };
                self.wires.add(q, corner);
                self.wires.add(corner, p2);
            }
        }
        self.repair();

        // Safeguard (not part of Logisim): if the re-routing made two pins of
        // this node share a net although they did not before (two jogs
        // overlapped and were merged), leave the wires behind instead.
        if let Some(snap) = snapshot {
            let after = self.pin_nets(&self.nodes[id].pins());
            if shorted(&before, &after) {
                *self = snap;
                self.move_node(id, x, y, false);
            }
        }
    }

    // ----- wires -----

    /// Wiring tool. Returns true when the drag shortened / deleted an
    /// existing wire instead of adding one.
    pub fn draw_wire(&mut self, a: Loc, b: Loc, horizontal_first: bool) -> bool {
        let segs = wiring::drag_segments(a, b, horizontal_first);
        if segs.len() == 1 && self.wires.shorten(segs[0].0, segs[0].1) {
            self.repair();
            return true;
        }
        for (p, q) in segs {
            self.wires.add(p, q);
        }
        self.repair();
        false
    }

    pub fn remove_wire(&mut self, w: &Wire) {
        self.wires.remove(w);
        self.repair();
    }

    /// Alt+drag: move one wire by (dx, dy). Whatever touched its ends stays
    /// connected through new wires. Returns the wire at its new position.
    pub fn move_wire(&mut self, w: &Wire, dx: i32, dy: i32) -> Option<Wire> {
        if dx == 0 && dy == 0 {
            return Some(*w);
        }
        let pins: HashSet<Loc> = self.pin_locs().into_iter().collect();
        let mut jobs: Vec<(Loc, bool)> = Vec::new(); // (old end, first leg horizontal)
        for e in [w.e0, w.e1] {
            let other = self.wires.iter().find(|o| *o != w && o.has_end(e));
            if other.is_some() || pins.contains(&e) {
                jobs.push((e, other.map_or(true, |o| o.horizontal())));
            }
        }
        self.wires.remove(w);
        let moved = Wire::new(
            (w.e0.0 + dx, w.e0.1 + dy),
            (w.e1.0 + dx, w.e1.1 + dy),
        );
        if let Some(m) = moved {
            self.wires.add(m.e0, m.e1);
        }
        for (e, horiz) in jobs {
            let e2 = (e.0 + dx, e.1 + dy);
            let corner = if horiz { (e2.0, e.1) } else { (e.0, e2.1) };
            self.wires.add(e, corner);
            self.wires.add(corner, e2);
        }
        self.repair();
        moved.and_then(|m| self.wires.containing(m.e0, m.e1))
    }

    // ----- simulation -----

    pub fn analyze(&self) -> Analysis {
        let points = self.pin_locs();
        let nets = Nets::build(&self.wires, &points);
        let count = nets.count();

        let mut drivers: Vec<Vec<Id>> = vec![Vec::new(); count];
        let mut sinks: Vec<Vec<Id>> = vec![Vec::new(); count];
        for n in &self.nodes {
            if let Some(k) = nets.net_at(n.output()) {
                drivers[k].push(n.id);
            }
            for p in n.inputs() {
                if let Some(k) = nets.net_at(p) {
                    sinks[k].push(n.id);
                }
            }
        }
        let state: Vec<NetState> = (0..count)
            .map(|k| match (drivers[k].len(), sinks[k].len()) {
                (0, 0) => NetState::Bare,
                (0, _) => NetState::Floating,
                (1, _) => NetState::Driven(drivers[k][0]),
                _ => NetState::Conflict,
            })
            .collect();

        // Node ids are indices, so circuit ids == node ids.
        let mut circuit = Circuit::new();
        for n in &self.nodes {
            match n.kind {
                Kind::Input => {
                    circuit.add_input(n.level);
                }
                Kind::Gate(g) => {
                    circuit.add_gate(g);
                }
            }
        }
        // Only singly-driven nets feed their sinks; floating and conflicting
        // nets leave the pin unwired, which simulates as Unknown.
        for n in &self.nodes {
            for p in n.inputs() {
                if let Some(k) = nets.net_at(p) {
                    if let NetState::Driven(d) = state[k] {
                        let _ = circuit.connect(d, n.id);
                    }
                }
            }
        }
        let (values, cycle) = circuit.simulate_partial();

        let mut on_cycle = vec![false; self.nodes.len()];
        for &id in &cycle {
            if id < on_cycle.len() {
                on_cycle[id] = true;
            }
        }
        let net_cycle: Vec<bool> = (0..count)
            .map(|k| {
                matches!(state[k], NetState::Driven(d) if on_cycle[d])
                    && sinks[k].iter().any(|&s| on_cycle[s])
            })
            .collect();

        Analysis { circuit, values, cycle, nets, state, net_cycle }
    }

    pub fn build(&self) -> Circuit {
        self.analyze().circuit
    }
}

fn shorted(before: &[usize], after: &[usize]) -> bool {
    for i in 0..before.len() {
        for j in (i + 1)..before.len() {
            if before[i] != before[j] && after[i] == after[j] {
                return true;
            }
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use logic::Gate;

    /// A(0,0) -> and.in0, B(0,60) -> and.in1, AND at (100,20).
    fn and_board() -> (Doc, Id, Id, Id) {
        let mut d = Doc::new();
        let a = d.add(Kind::Input, 0, 0); // out (50,30)
        let b = d.add(Kind::Input, 0, 60); // out (50,90)
        let g = d.add(Kind::Gate(Gate::And), 100, 20); // in (100,40) (100,60), out (160,50)
        d.draw_wire((50, 30), (80, 40), true);
        d.draw_wire((80, 40), (100, 40), true);
        d.draw_wire((50, 90), (80, 60), true);
        d.draw_wire((80, 60), (100, 60), true);
        (d, a, b, g)
    }

    #[test]
    fn signal_flows_through_free_wires() {
        let (mut d, a, b, g) = and_board();
        assert_eq!(d.analyze().values[g], Value::Low);
        d.toggle(a);
        d.toggle(b);
        let an = d.analyze();
        assert_eq!(an.values[g], Value::High);
        assert_eq!(an.look(d.node(g).output()), Look::Value(Value::High));
    }

    #[test]
    fn moving_along_the_wires_keeps_connections() {
        let (mut d, a, b, g) = and_board();
        d.toggle(a);
        d.toggle(b);
        d.move_node(g, 120, 20, true);
        assert_eq!(d.analyze().values[g], Value::High);
        d.move_node(g, 140, 20, false); // Shift: wires stay behind
        assert_eq!(d.analyze().values[g], Value::Unknown);
    }

    #[test]
    fn two_outputs_on_one_net_conflict() {
        let mut d = Doc::new();
        d.add(Kind::Input, 0, 0);
        d.add(Kind::Input, 0, 60);
        d.draw_wire((50, 30), (50, 90), true);
        assert_eq!(d.analyze().look((50, 30)), Look::Conflict);
    }

    #[test]
    fn a_dropped_pin_connects_to_a_wire_it_lands_on() {
        let mut d = Doc::new();
        let a = d.add(Kind::Input, 0, 0);
        d.toggle(a);
        d.draw_wire((50, 30), (150, 30), true);
        let n = d.add(Kind::Gate(Gate::Not), 100, 0); // input pin (100,30) is mid-wire
        assert_eq!(d.analyze().values[n], Value::Low);
    }

    #[test]
    fn feedback_is_flagged() {
        let mut d = Doc::new();
        let n = d.add(Kind::Gate(Gate::Not), 0, 0); // in (0,30), out (60,30)
        d.draw_wire((60, 30), (80, 30), true);
        d.draw_wire((80, 30), (-20, 60), false);
        d.draw_wire((-20, 60), (-20, 30), true);
        d.draw_wire((-20, 30), (0, 30), true);
        let _ = n;
        assert_eq!(d.analyze().look((60, 30)), Look::Cycle);
    }
}
