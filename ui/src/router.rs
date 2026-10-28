//! A* router for orthogonal wires on the CELL grid.
//!
//! Wires must run horizontally or vertically, never diagonally, and the
//! router's job is to find the shortest such path between two port
//! positions that (a) avoids every node rectangle and (b) changes
//! direction as few times as possible. This is the same idea Logisim
//! uses, and it is the difference between "straight line with a random
//! dogleg" and "a wire that goes around the gate instead of through it".
//!
//! The search state is `(cell, direction)` rather than just `cell`: the
//! direction the router entered the cell from is what lets `BEND_COST`
//! count the turns, so the resulting path has as few corners as possible.
//! A soft fallback (`fallback`) keeps the editor usable if the A* search
//! fails — for instance when the two ports are surrounded by nodes.

use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap};

use crate::shapes;

const STRAIGHT_COST: i32 = 10;
const BEND_COST: i32 = 80;
const HEURISTIC_WEIGHT: i32 = 10;
const PAD: i32 = 4;
const MAX_GRID: i32 = 120;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
struct State {
    x: i32,
    y: i32,
    d: u8,
}

#[derive(Clone, Copy, PartialEq, Eq)]
struct Item {
    f: i32,
    g: i32,
    s: State,
}

impl Ord for Item {
    fn cmp(&self, other: &Self) -> Ordering {
        other.f.cmp(&self.f)
    }
}
impl PartialOrd for Item {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// Route between two world-space points, avoiding the given node
/// rectangles. Returns the interior corners only — the caller adds the
/// two port positions at either end.
pub fn route(
    start: (f64, f64),
    end: (f64, f64),
    obstacles: &[(f64, f64, f64, f64)],
) -> Vec<(f64, f64)> {
    let cell = shapes::CELL;
    let to_cell = |p: (f64, f64)| -> (i32, i32) {
        ((p.0 / cell).round() as i32, (p.1 / cell).round() as i32)
    };
    let sc = to_cell(start);
    let ec = to_cell(end);

    if sc == ec {
        return vec![];
    }
    if sc.0 == ec.0 || sc.1 == ec.1 {
        return vec![];
    }

    let min_x = sc.0.min(ec.0) - PAD;
    let max_x = sc.0.max(ec.0) + PAD;
    let min_y = sc.1.min(ec.1) - PAD;
    let max_y = sc.1.max(ec.1) + PAD;
    let w = max_x - min_x + 1;
    let h = max_y - min_y + 1;
    if w <= 0 || h <= 0 || w > MAX_GRID || h > MAX_GRID {
        return fallback(start, end);
    }

    let mut blocked = vec![false; (w * h) as usize];
    for &(ox, oy, ow, oh) in obstacles {
        let (cx0, cy0) = to_cell((ox, oy));
        let (cx1, cy1) = to_cell((ox + ow, oy + oh));
        for cy in cy0..=cy1 {
            for cx in cx0..=cx1 {
                let lx = cx - min_x;
                let ly = cy - min_y;
                if lx >= 0 && ly >= 0 && lx < w && ly < h {
                    blocked[(ly * w + lx) as usize] = true;
                }
            }
        }
    }
    let idx = |x: i32, y: i32| -> Option<usize> {
        let lx = x - min_x;
        let ly = y - min_y;
        if lx >= 0 && ly >= 0 && lx < w && ly < h {
            Some((ly * w + lx) as usize)
        } else {
            None
        }
    };
    if let Some(i) = idx(sc.0, sc.1) {
        blocked[i] = false;
    }
    if let Some(i) = idx(ec.0, ec.1) {
        blocked[i] = false;
    }

    let heur = |s: State| -> i32 {
        ((s.x - ec.0).abs() + (s.y - ec.1).abs()) * HEURISTIC_WEIGHT
    };
    let dirs: [(i32, i32, u8); 4] = [(1, 0, 0), (0, 1, 1), (-1, 0, 2), (0, -1, 3)];

    let mut open: BinaryHeap<Item> = BinaryHeap::new();
    let mut g_best: HashMap<State, i32> = HashMap::new();
    let mut came: HashMap<State, State> = HashMap::new();

    for &(dx, dy, d) in &dirs {
        let nx = sc.0 + dx;
        let ny = sc.1 + dy;
        if let Some(i) = idx(nx, ny) {
            if !blocked[i] {
                let s = State { x: nx, y: ny, d };
                g_best.insert(s, STRAIGHT_COST);
                open.push(Item {
                    f: STRAIGHT_COST + heur(s),
                    g: STRAIGHT_COST,
                    s,
                });
            }
        }
    }

    let mut solution: Option<State> = None;
    while let Some(item) = open.pop() {
        if item.s.x == ec.0 && item.s.y == ec.1 {
            solution = Some(item.s);
            break;
        }
        if item.g > *g_best.get(&item.s).unwrap_or(&i32::MAX) {
            continue;
        }
        let pd = dirs[item.s.d as usize];
        for &(dx, dy, d) in &dirs {
            if dx == -pd.0 && dy == -pd.1 {
                continue;
            }
            let nx = item.s.x + dx;
            let ny = item.s.y + dy;
            let ni = match idx(nx, ny) {
                Some(i) => i,
                None => continue,
            };
            if blocked[ni] {
                continue;
            }
            let step = STRAIGHT_COST + if d != item.s.d { BEND_COST } else { 0 };
            let ng = item.g + step;
            let ns = State { x: nx, y: ny, d };
            if ng < *g_best.get(&ns).unwrap_or(&i32::MAX) {
                g_best.insert(ns, ng);
                came.insert(ns, item.s);
                open.push(Item {
                    f: ng + heur(ns),
                    g: ng,
                    s: ns,
                });
            }
        }
    }

    let end_s = match solution {
        Some(s) => s,
        None => return fallback(start, end),
    };

    let mut states = Vec::new();
    let mut cur = end_s;
    loop {
        states.push(cur);
        match came.get(&cur) {
            Some(&p) => cur = p,
            None => break,
        }
    }
    states.reverse();

    let mut corners: Vec<(f64, f64)> = Vec::new();
    for i in 0..states.len().saturating_sub(1) {
        if states[i].d != states[i + 1].d {
            corners.push((states[i].x as f64 * cell, states[i].y as f64 * cell));
        }
    }
    let near = |a: (f64, f64), b: (f64, f64)| {
        (a.0 - b.0).abs() < cell / 2.0 && (a.1 - b.1).abs() < cell / 2.0
    };
    corners.retain(|&c| !near(c, start) && !near(c, end));
    corners
}

fn fallback(start: (f64, f64), end: (f64, f64)) -> Vec<(f64, f64)> {
    let cell = shapes::CELL;
    let (sx, sy) = start;
    let (dx, dy) = end;
    if (dx - sx).abs() < 0.01 || (dy - sy).abs() < 0.01 {
        return vec![];
    }
    if dx >= sx + 2.0 * cell {
        let mid = (sx + cell).max((dx - cell).min(((sx + dx) / 2.0).round() / cell * cell));
        vec![(mid, sy), (mid, dy)]
    } else {
        let top = sy.min(dy) - cell;
        vec![(sx + cell, sy), (sx + cell, top), (dx, top)]
    }
}
