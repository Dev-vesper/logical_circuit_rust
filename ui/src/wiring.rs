//! Logisim's wire model.
//!
//! * A wire is an axis-aligned segment between two grid points. It has no
//!   idea what it is attached to; it is non-directional.
//! * Connectivity is purely geometric: two things are connected when they
//!   share a coordinate (`CircuitWires.connectWires` in Logisim: wires unite
//!   their two end points, pins attach by location).
//! * The wire set is kept normalised ("repaired") after every edit, so that
//!   wires never overlap, are split wherever a pin or another wire's end
//!   touches their interior, and are merged where two collinear wires meet
//!   at a point with nothing else on it.
//! * A dot is drawn where more than two things meet at a point.

use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};

pub type Loc = (i32, i32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Wire {
    pub e0: Loc,
    pub e1: Loc,
}

impl Wire {
    /// `None` for zero-length or diagonal segments. Ends are stored sorted.
    pub fn new(a: Loc, b: Loc) -> Option<Wire> {
        if a == b || (a.0 != b.0 && a.1 != b.1) {
            return None;
        }
        Some(if a <= b {
            Wire { e0: a, e1: b }
        } else {
            Wire { e0: b, e1: a }
        })
    }

    pub fn horizontal(&self) -> bool {
        self.e0.1 == self.e1.1
    }

    pub fn has_end(&self, p: Loc) -> bool {
        p == self.e0 || p == self.e1
    }

    pub fn other_end(&self, p: Loc) -> Loc {
        if p == self.e0 {
            self.e1
        } else {
            self.e0
        }
    }

    /// Includes both ends.
    pub fn contains(&self, p: Loc) -> bool {
        if self.horizontal() {
            p.1 == self.e0.1 && self.e0.0 <= p.0 && p.0 <= self.e1.0
        } else {
            p.0 == self.e0.0 && self.e0.1 <= p.1 && p.1 <= self.e1.1
        }
    }

    pub fn strictly_inside(&self, p: Loc) -> bool {
        self.contains(p) && !self.has_end(p)
    }
}

/// The segments a drag from `start` to `end` creates: one wire if the two
/// points share a row or column, otherwise an L made of two wires. Which leg
/// comes first is chosen by the caller from the initial mouse movement.
pub fn drag_segments(start: Loc, end: Loc, horizontal_first: bool) -> Vec<(Loc, Loc)> {
    if start == end {
        return vec![];
    }
    if start.0 == end.0 || start.1 == end.1 {
        return vec![(start, end)];
    }
    let corner = if horizontal_first {
        (end.0, start.1)
    } else {
        (start.0, end.1)
    };
    vec![(start, corner), (corner, end)]
}

#[derive(Debug, Clone, Default)]
pub struct Wires {
    set: BTreeSet<Wire>,
}

impl Wires {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn iter(&self) -> impl Iterator<Item = &Wire> {
        self.set.iter()
    }

    pub fn len(&self) -> usize {
        self.set.len()
    }

    pub fn is_empty(&self) -> bool {
        self.set.is_empty()
    }

    pub fn contains(&self, w: &Wire) -> bool {
        self.set.contains(w)
    }

    pub fn to_vec(&self) -> Vec<Wire> {
        self.set.iter().copied().collect()
    }

    pub fn clear(&mut self) {
        self.set.clear();
    }

    /// Raw insert; call `repair` afterwards.
    pub fn add(&mut self, a: Loc, b: Loc) -> bool {
        match Wire::new(a, b) {
            Some(w) => {
                self.set.insert(w);
                true
            }
            None => false,
        }
    }

    pub fn remove(&mut self, w: &Wire) -> bool {
        self.set.remove(w)
    }

    /// The wire that contains both points, if any.
    pub fn containing(&self, a: Loc, b: Loc) -> Option<Wire> {
        self.set
            .iter()
            .copied()
            .find(|w| w.contains(a) && w.contains(b))
    }

    // ---------- shortening ----------

    /// A drag that starts or ends at a terminus of an existing wire and lies
    /// entirely on it removes that part of the wire instead of adding one.
    /// Returns (wire, terminus, inner point).
    fn shorten_plan(&self, a: Loc, b: Loc) -> Option<(Wire, Loc, Loc)> {
        if a == b {
            return None;
        }
        let w = self
            .set
            .iter()
            .copied()
            .find(|w| w.contains(a) && w.contains(b) && (w.has_end(a) || w.has_end(b)))?;
        Some(if w.has_end(a) { (w, a, b) } else { (w, b, a) })
    }

    /// The part that `shorten` would remove (for the white preview line).
    pub fn shortening(&self, a: Loc, b: Loc) -> Option<Wire> {
        let (_, term, inner) = self.shorten_plan(a, b)?;
        Wire::new(term, inner)
    }

    pub fn shorten(&mut self, a: Loc, b: Loc) -> bool {
        match self.shorten_plan(a, b) {
            None => false,
            Some((w, term, inner)) => {
                self.set.remove(&w);
                if let Some(rest) = Wire::new(inner, w.other_end(term)) {
                    self.set.insert(rest);
                }
                true
            }
        }
    }

    // ---------- repair ----------

    /// Restore the invariants. `pins` are all component pin locations.
    pub fn repair(&mut self, pins: &HashSet<Loc>) {
        loop {
            let mut changed = false;
            changed |= self.merge_overlaps();
            changed |= self.split_at_points(pins);
            changed |= self.merge_touching(pins);
            if !changed {
                break;
            }
        }
    }

    /// Collinear wires that share more than a point become one wire.
    fn merge_overlaps(&mut self) -> bool {
        // key: (is_horizontal, fixed coordinate) -> intervals along the line
        let mut rows: BTreeMap<(bool, i32), Vec<(i32, i32)>> = BTreeMap::new();
        for w in &self.set {
            if w.horizontal() {
                rows.entry((true, w.e0.1)).or_default().push((w.e0.0, w.e1.0));
            } else {
                rows.entry((false, w.e0.0)).or_default().push((w.e0.1, w.e1.1));
            }
        }
        let mut out = BTreeSet::new();
        for ((horiz, fixed), mut ivs) in rows {
            ivs.sort();
            let mut cur = ivs[0];
            for &(lo, hi) in &ivs[1..] {
                if lo < cur.1 {
                    cur.1 = cur.1.max(hi);
                } else {
                    push_interval(&mut out, horiz, fixed, cur.0, cur.1);
                    cur = (lo, hi);
                }
            }
            push_interval(&mut out, horiz, fixed, cur.0, cur.1);
        }
        let changed = out != self.set;
        self.set = out;
        changed
    }

    /// A pin or another wire's end lying inside a wire splits it there.
    fn split_at_points(&mut self, pins: &HashSet<Loc>) -> bool {
        let mut points: BTreeSet<Loc> = pins.iter().copied().collect();
        for w in &self.set {
            points.insert(w.e0);
            points.insert(w.e1);
        }
        let mut out = BTreeSet::new();
        let mut changed = false;
        for w in &self.set {
            let cuts: Vec<Loc> = points
                .iter()
                .copied()
                .filter(|&p| w.strictly_inside(p))
                .collect(); // BTreeSet order == order along the wire
            if cuts.is_empty() {
                out.insert(*w);
                continue;
            }
            changed = true;
            let mut prev = w.e0;
            for c in cuts {
                out.insert(Wire { e0: prev, e1: c });
                prev = c;
            }
            out.insert(Wire { e0: prev, e1: w.e1 });
        }
        self.set = out;
        changed
    }

    /// Two collinear wires meeting at a point with no pin and no third wire
    /// are one wire.
    fn merge_touching(&mut self, pins: &HashSet<Loc>) -> bool {
        let mut any = false;
        loop {
            let mut ends: HashMap<Loc, Vec<Wire>> = HashMap::new();
            for w in &self.set {
                ends.entry(w.e0).or_default().push(*w);
                ends.entry(w.e1).or_default().push(*w);
            }
            let found = ends.iter().find_map(|(p, ws)| {
                if ws.len() == 2 && !pins.contains(p) && ws[0].horizontal() == ws[1].horizontal() {
                    Some((ws[0], ws[1]))
                } else {
                    None
                }
            });
            match found {
                Some((a, b)) => {
                    self.set.remove(&a);
                    self.set.remove(&b);
                    self.set.insert(Wire {
                        e0: a.e0.min(b.e0),
                        e1: a.e1.max(b.e1),
                    });
                    any = true;
                }
                None => break,
            }
        }
        any
    }
}

fn push_interval(out: &mut BTreeSet<Wire>, horiz: bool, fixed: i32, lo: i32, hi: i32) {
    out.insert(if horiz {
        Wire { e0: (lo, fixed), e1: (hi, fixed) }
    } else {
        Wire { e0: (fixed, lo), e1: (fixed, hi) }
    });
}

// ---------- nets ----------

pub type NetId = usize;

/// Connected components over locations (Logisim's `BundleMap`, one bit wide).
#[derive(Debug, Clone, Default)]
pub struct Nets {
    net_of: HashMap<Loc, NetId>,
    count: usize,
}

fn id_of(index: &mut HashMap<Loc, usize>, p: Loc) -> usize {
    let n = index.len();
    *index.entry(p).or_insert(n)
}

fn find(parent: &mut Vec<usize>, mut x: usize) -> usize {
    while parent[x] != x {
        parent[x] = parent[parent[x]];
        x = parent[x];
    }
    x
}

impl Nets {
    /// `points` are extra locations (pins) that must belong to some net even
    /// when no wire touches them: two pins on the same spot are connected.
    pub fn build(wires: &Wires, points: &[Loc]) -> Nets {
        let mut index: HashMap<Loc, usize> = HashMap::new();
        for &p in points {
            id_of(&mut index, p);
        }
        for w in wires.iter() {
            id_of(&mut index, w.e0);
            id_of(&mut index, w.e1);
        }
        let mut parent: Vec<usize> = (0..index.len()).collect();
        for w in wires.iter() {
            let ra = find(&mut parent, index[&w.e0]);
            let rb = find(&mut parent, index[&w.e1]);
            if ra != rb {
                parent[ra] = rb;
            }
        }
        let mut dense: HashMap<usize, usize> = HashMap::new();
        let mut net_of = HashMap::new();
        for (&loc, &i) in &index {
            let root = find(&mut parent, i);
            let next = dense.len();
            let id = *dense.entry(root).or_insert(next);
            net_of.insert(loc, id);
        }
        Nets { net_of, count: dense.len() }
    }

    pub fn net_at(&self, p: Loc) -> Option<NetId> {
        self.net_of.get(&p).copied()
    }

    pub fn count(&self) -> usize {
        self.count
    }
}

/// Points where more than two things meet (wire ends and pins both count).
pub fn junctions(wires: &Wires, pins: &[Loc]) -> Vec<Loc> {
    let mut count: HashMap<Loc, usize> = HashMap::new();
    for w in wires.iter() {
        *count.entry(w.e0).or_default() += 1;
        *count.entry(w.e1).or_default() += 1;
    }
    for &p in pins {
        *count.entry(p).or_default() += 1;
    }
    count
        .into_iter()
        .filter(|&(_, c)| c > 2)
        .map(|(p, _)| p)
        .collect()
}
