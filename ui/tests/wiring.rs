//! Integration tests for the wire model. Only `Wires`, `Wire`, `Nets` and
//! `drag_segments` are used — all of them public API of `ui::wiring`.

use std::collections::HashSet;

use ui::wiring::{drag_segments, Loc, Nets, Wire, Wires};

fn pins(v: &[Loc]) -> HashSet<Loc> {
    v.iter().copied().collect()
}

fn w(a: Loc, b: Loc) -> Wire {
    Wire::new(a, b).unwrap()
}

#[test]
fn collinear_touching_wires_merge() {
    let mut s = Wires::new();
    s.add((0, 0), (10, 0));
    s.add((10, 0), (30, 0));
    s.repair(&pins(&[]));
    assert_eq!(s.to_vec(), vec![w((0, 0), (30, 0))]);
}

#[test]
fn a_pin_keeps_the_split() {
    let mut s = Wires::new();
    s.add((0, 0), (30, 0));
    s.repair(&pins(&[(10, 0)]));
    assert_eq!(s.to_vec(), vec![w((0, 0), (10, 0)), w((10, 0), (30, 0))]);
}

#[test]
fn a_wire_end_in_the_middle_splits_the_host() {
    let mut s = Wires::new();
    s.add((0, 0), (30, 0));
    s.add((10, 0), (10, 20));
    s.repair(&pins(&[]));
    assert_eq!(s.len(), 3);
    assert!(s.contains(&w((0, 0), (10, 0))));
    assert!(s.contains(&w((10, 0), (30, 0))));
}

#[test]
fn overlap_becomes_a_union() {
    let mut s = Wires::new();
    s.add((0, 0), (20, 0));
    s.add((10, 0), (40, 0));
    s.repair(&pins(&[]));
    assert_eq!(s.to_vec(), vec![w((0, 0), (40, 0))]);
}

#[test]
fn shortening_and_deleting() {
    let mut s = Wires::new();
    s.add((0, 0), (30, 0));
    assert!(s.shorten((0, 0), (10, 0)));
    assert_eq!(s.to_vec(), vec![w((10, 0), (30, 0))]);
    assert!(s.shorten((30, 0), (10, 0)));
    assert!(s.is_empty());
}

#[test]
fn nets_follow_shared_coordinates() {
    let mut s = Wires::new();
    s.add((0, 0), (10, 0));
    s.add((10, 0), (10, 10));
    let n = Nets::build(&s, &[(50, 50)]);
    assert_eq!(n.net_at((0, 0)), n.net_at((10, 10)));
    assert_ne!(n.net_at((0, 0)), n.net_at((50, 50)));
    assert_eq!(n.count(), 2);
}

#[test]
fn l_drag_follows_the_chosen_leg() {
    assert_eq!(
        drag_segments((0, 0), (20, 10), true),
        vec![((0, 0), (20, 0)), ((20, 0), (20, 10))]
    );
    assert_eq!(
        drag_segments((0, 0), (20, 10), false),
        vec![((0, 0), (0, 10)), ((0, 10), (20, 10))]
    );
}
