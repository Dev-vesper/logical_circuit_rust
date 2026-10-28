//! The board widget: dot grid, symbols, wires, live values, and the gestures.
//!
//! The gestures follow Logisim's Edit tool:
//!  * over a pin, a wire end, or a point on a wire a small green circle
//!    shows; pressing there and dragging draws a wire (an L when the drag is
//!    diagonal; its first leg is fixed by the initial movement);
//!  * released without dragging, the press just selects the wire;
//!  * a drag that starts at a wire's end and runs back over it shortens or
//!    deletes that wire (shown by a white line);
//!  * Alt on the middle of a wire: the green circle disappears, press selects
//!    the wire and a drag moves it; Alt on empty space starts a wire anyway;
//!  * dragging a node moves it and keeps its connections; Shift disables that.

use std::cell::RefCell;
use std::rc::Rc;

use gtk::prelude::*;
use logic::{Circuit, Kind, Value};

use crate::doc::{Analysis, Doc, Id, Look};
use crate::layout;
use crate::shapes;
use crate::wiring::{self, Loc, Wire};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tool {
    Select,
    Place(Kind),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Selection {
    Node(Id),
    Wire(Wire),
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Mods {
    pub shift: bool,
    pub alt: bool,
}

#[derive(Debug, Clone)]
pub enum Drag {
    /// Dragging a node. `base` is the document at press time; every motion
    /// re-applies the move to a fresh copy, so the preview is exactly the
    /// final result and Escape can restore it.
    Moving {
        node: Id,
        offset: (f64, f64),
        origin: (f64, f64),
        base: Doc,
        moved: bool,
    },
    MovingWire {
        wire: Wire,
        origin: (f64, f64),
        base: Doc,
        moved: bool,
    },
    Wiring {
        start: Loc,
        /// The wire the press landed on, selected if this turns out to be a click.
        on_wire: Option<Wire>,
        origin: (f64, f64),
        cur: Loc,
        first_horizontal: Option<bool>,
        dragged: bool,
    },
}

pub struct View<'a> {
    pub analysis: &'a Analysis,
    pub letters: &'a [String],
    pub selection: Option<Selection>,
    pub hint: Option<Loc>,
    pub drag: Option<&'a Drag>,
}

/// How close the mouse must be to a pin / wire end to start a wire there.
const WIRE_START_RADIUS: f64 = 8.0;
/// How close the mouse must be to a wire to count as "on" it.
const WIRE_HIT_RADIUS: f64 = 5.0;
const DRAG_THRESHOLD: f64 = 4.0;
const DOT_SPACING: f64 = layout::CELL as f64;

thread_local! {
    static GRID_CACHE: RefCell<Option<(i32, i32, cairo::ImageSurface)>> = RefCell::new(None);
}

fn paint_grid(cr: &cairo::Context, width: f64, height: f64) {
    let w = width.ceil() as i32;
    let h = height.ceil() as i32;
    if w <= 0 || h <= 0 {
        return;
    }
    GRID_CACHE.with(|cache| {
        {
            let needs_rebuild = match &*cache.borrow() {
                Some((cw, ch, _)) => *cw != w || *ch != h,
                None => true,
            };
            if needs_rebuild {
                let surface = cairo::ImageSurface::create(cairo::Format::ARgb32, w, h)
                    .expect("grid surface");
                {
                    let gcr = cairo::Context::new(&surface).expect("grid context");
                    shapes::set_color(&gcr, shapes::PAPER);
                    let _ = gcr.paint();
                    shapes::set_color(&gcr, shapes::GRID);
                    let mut y = 0.0;
                    while y < height {
                        let mut x = 0.0;
                        while x < width {
                            gcr.arc(x, y, 0.8, 0.0, 2.0 * std::f64::consts::PI);
                            let _ = gcr.fill();
                            x += DOT_SPACING;
                        }
                        y += DOT_SPACING;
                    }
                }
                *cache.borrow_mut() = Some((w, h, surface));
            }
        }
        let cache_ref = cache.borrow();
        if let Some((_, _, surface)) = &*cache_ref {
            let _ = cr.set_source_surface(surface, 0.0, 0.0);
            let _ = cr.paint();
        }
    });
}

fn letter(k: usize) -> String {
    if k < 26 {
        ((b'A' + k as u8) as char).to_string()
    } else {
        format!("in{}", k)
    }
}

fn name_of(kind: &Kind) -> &'static str {
    match kind {
        Kind::Input => "input",
        Kind::Gate(g) => g.name(),
    }
}

fn letters_of(doc: &Doc) -> Vec<String> {
    let mut letters = vec![String::new(); doc.count()];
    let mut k = 0;
    doc.iter(|n| {
        if matches!(n.kind, Kind::Input) {
            letters[n.id] = letter(k);
            k += 1;
        }
    });
    letters
}

fn value_of(a: &Analysis, id: Id) -> Value {
    a.values.get(id).copied().unwrap_or(Value::Unknown)
}

fn look_color(l: Look) -> (f64, f64, f64) {
    match l {
        Look::Bare => shapes::NIL,
        Look::Value(v) => shapes::color_of_value(v),
        Look::Conflict | Look::Cycle => shapes::ERROR,
    }
}

fn distance_to_segment(px: f64, py: f64, ax: f64, ay: f64, bx: f64, by: f64) -> f64 {
    let vx = bx - ax;
    let vy = by - ay;
    let length2 = vx * vx + vy * vy;
    let t = if length2 == 0.0 {
        0.0
    } else {
        (0.0_f64).max(1.0_f64.min(((px - ax) * vx + (py - ay) * vy) / length2))
    };
    ((px - (ax + t * vx)).powi(2) + (py - (ay + t * vy)).powi(2)).sqrt()
}

fn dist(a: (f64, f64), b: (f64, f64)) -> f64 {
    ((a.0 - b.0).powi(2) + (a.1 - b.1).powi(2)).sqrt()
}

fn stroke_line(cr: &cairo::Context, color: (f64, f64, f64), width: f64, a: Loc, b: Loc) {
    cr.move_to(a.0 as f64, a.1 as f64);
    cr.line_to(b.0 as f64, b.1 as f64);
    shapes::set_color(cr, color);
    cr.set_line_width(width);
    cr.set_line_cap(cairo::LineCap::Square);
    let _ = cr.stroke();
}

fn dot(cr: &cairo::Context, p: Loc, radius: f64, color: (f64, f64, f64)) {
    cr.arc(p.0 as f64, p.1 as f64, radius, 0.0, 2.0 * std::f64::consts::PI);
    shapes::set_color(cr, color);
    let _ = cr.fill();
}

// ---------- rendering ----------

pub fn render(cr: &cairo::Context, doc: &Doc, width: f64, height: f64, view: &View<'_>) {
    paint_grid(cr, width, height);
    let a = view.analysis;

    for w in doc.wires().iter() {
        if view.selection == Some(Selection::Wire(*w)) {
            stroke_line(cr, shapes::SELECTED, 7.0, w.e0, w.e1);
        }
        stroke_line(cr, look_color(a.look(w.e0)), 3.0, w.e0, w.e1);
    }

    doc.iter(|n| {
        let (x, y) = (n.x as f64, n.y as f64);
        match n.kind {
            Kind::Input => shapes::draw_input(
                cr,
                x,
                y,
                &view.letters[n.id],
                if n.level { Value::High } else { Value::Low },
            ),
            Kind::Gate(g) => shapes::draw_gate(cr, x, y, g),
        }
        for p in n.pins() {
            dot(cr, p, 3.0, look_color(a.look(p)));
        }
        if view.selection == Some(Selection::Node(n.id)) {
            let s = layout::SIZE as f64;
            cr.rectangle(x - 6.0, y - 6.0, s + 12.0, s + 12.0);
            shapes::set_color(cr, shapes::SELECTED);
            cr.set_line_width(2.0);
            cr.set_dash(&[5.0, 3.0], 0.0);
            let _ = cr.stroke();
            cr.set_dash(&[], 0.0);
        }
    });

    // Junction dots: more than two things meet here.
    for p in wiring::junctions(doc.wires(), &doc.pin_locs()) {
        dot(cr, p, 4.0, look_color(a.look(p)));
    }

    if let Some(p) = view.hint {
        cr.arc(p.0 as f64, p.1 as f64, 6.0, 0.0, 2.0 * std::f64::consts::PI);
        shapes::set_color(cr, shapes::HINT);
        cr.set_line_width(2.0);
        let _ = cr.stroke();
    }

    if let Some(Drag::Wiring {
        start,
        cur,
        first_horizontal,
        dragged: true,
        ..
    }) = view.drag
    {
        let segs = wiring::drag_segments(*start, *cur, first_horizontal.unwrap_or(true));
        let removed = if segs.len() == 1 {
            doc.wires().shortening(segs[0].0, segs[0].1)
        } else {
            None
        };
        match removed {
            // Shortening: a white line over the part that will disappear.
            Some(part) => stroke_line(cr, shapes::PAPER, 5.0, part.e0, part.e1),
            None => {
                for (p, q) in &segs {
                    stroke_line(cr, shapes::INK, 3.0, *p, *q);
                }
            }
        }
        dot(cr, *cur, 3.0, shapes::HINT);
    }
}

pub fn render_board(cr: &cairo::Context, doc: &Doc, width: f64, height: f64) {
    let analysis = doc.analyze();
    let letters = letters_of(doc);
    let view = View {
        analysis: &analysis,
        letters: &letters,
        selection: None,
        hint: None,
        drag: None,
    };
    render(cr, doc, width, height, &view);
}

// ---------- the widget ----------

struct CanvasState {
    doc: Doc,
    analysis: Analysis,
    letters: Vec<String>,
    tool: Tool,
    selection: Option<Selection>,
    hint: Option<Loc>,
    drag: Option<Drag>,
    report: Box<dyn FnMut(String)>,
    tool_used: Box<dyn FnMut(Tool)>,
}

impl CanvasState {
    fn new() -> Self {
        let doc = Doc::new();
        CanvasState {
            analysis: doc.analyze(),
            letters: Vec::new(),
            doc,
            tool: Tool::Select,
            selection: None,
            hint: None,
            drag: None,
            report: Box::new(|_| {}),
            tool_used: Box::new(|_| {}),
        }
    }

    /// Recompute values/colours and drop a selection that no longer exists.
    fn refresh(&mut self) {
        self.analysis = self.doc.analyze();
        self.letters = letters_of(&self.doc);
        self.selection = match self.selection {
            Some(Selection::Node(id)) if id < self.doc.count() => Some(Selection::Node(id)),
            Some(Selection::Wire(w)) => self.doc.wires().contains(&w).then_some(Selection::Wire(w)),
            _ => None,
        };
    }

    fn view(&self) -> View<'_> {
        View {
            analysis: &self.analysis,
            letters: &self.letters,
            selection: self.selection,
            hint: self.hint,
            drag: self.drag.as_ref(),
        }
    }

    fn set_tool(&mut self, t: Tool) {
        if self.tool != t {
            self.tool = t;
            (self.tool_used)(t);
        }
    }

    // ----- hit testing -----

    fn node_at(&self, mx: f64, my: f64) -> Option<Id> {
        let s = layout::SIZE as f64;
        let mut hit = None;
        self.doc.iter(|n| {
            let (x, y) = (n.x as f64, n.y as f64);
            if mx >= x && mx <= x + s && my >= y && my <= y + s {
                hit = Some(n.id);
            }
        });
        hit
    }

    fn wire_at(&self, mx: f64, my: f64) -> Option<Wire> {
        let mut best: Option<(Wire, f64)> = None;
        for w in self.doc.wires().iter() {
            let d = distance_to_segment(
                mx,
                my,
                w.e0.0 as f64,
                w.e0.1 as f64,
                w.e1.0 as f64,
                w.e1.1 as f64,
            );
            if d <= WIRE_HIT_RADIUS && best.map_or(true, |(_, bd)| d < bd) {
                best = Some((*w, d));
            }
        }
        best.map(|(w, _)| w)
    }

    /// Nearest pin or wire end within `WIRE_START_RADIUS`.
    fn connection_near(&self, mx: f64, my: f64) -> Option<Loc> {
        let mut best: Option<(Loc, f64)> = None;
        let mut consider = |p: Loc| {
            let d = dist((mx, my), (p.0 as f64, p.1 as f64));
            if d <= WIRE_START_RADIUS && best.map_or(true, |(_, bd)| d < bd) {
                best = Some((p, d));
            }
        };
        for p in self.doc.pin_locs() {
            consider(p);
        }
        for w in self.doc.wires().iter() {
            consider(w.e0);
            consider(w.e1);
        }
        best.map(|(p, _)| p)
    }

    /// Where a wire would start if the button were pressed here (this is
    /// also where the green circle is drawn), plus the wire under the mouse.
    fn wire_start(&self, mx: f64, my: f64, alt: bool) -> Option<(Loc, Option<Wire>)> {
        let over = self.wire_at(mx, my);
        if let Some(p) = self.connection_near(mx, my) {
            return Some((p, over));
        }
        match over {
            // Atop a wire: start at the nearest grid point on it.
            Some(w) if !alt => {
                let p = if w.horizontal() {
                    (layout::snap_f(mx).clamp(w.e0.0, w.e1.0), w.e0.1)
                } else {
                    (w.e0.0, layout::snap_f(my).clamp(w.e0.1, w.e1.1))
                };
                Some((p, Some(w)))
            }
            // Alt in the middle of a wire: no circle, the press selects/moves it.
            Some(_) => None,
            // Alt outside every component: start a wire anywhere.
            None if alt && self.node_at(mx, my).is_none() => {
                Some(((layout::snap_f(mx), layout::snap_f(my)), None))
            }
            None => None,
        }
    }

    // ----- actions -----

    fn delete_selection(&mut self) {
        match self.selection {
            Some(Selection::Node(id)) => {
                self.doc.remove(id);
                (self.report)("node deleted".to_string());
            }
            Some(Selection::Wire(w)) => {
                self.doc.remove_wire(&w);
                (self.report)("wire deleted".to_string());
            }
            None => (self.report)("nothing is selected".to_string()),
        }
        self.selection = None;
        self.hint = None;
        self.refresh();
    }

    fn clear(&mut self) {
        self.doc.clear();
        self.selection = None;
        self.hint = None;
        self.drag = None;
        (self.report)("the board is empty".to_string());
        self.refresh();
    }

    // ----- gestures -----

    fn press(&mut self, button: u32, mods: Mods, mx: f64, my: f64) -> bool {
        self.hint = None;

        if button == 3 {
            // Right click cancels a wire being drawn, otherwise deletes.
            if matches!(self.drag, Some(Drag::Wiring { .. })) {
                self.drag = None;
                return true;
            }
            if self.drag.is_some() {
                return false;
            }
            if let Some(id) = self.node_at(mx, my) {
                self.doc.remove(id);
                self.selection = None;
                (self.report)("node deleted".to_string());
            } else if let Some(w) = self.wire_at(mx, my) {
                self.doc.remove_wire(&w);
                self.selection = None;
                (self.report)("wire deleted".to_string());
            }
            self.refresh();
            return true;
        }
        if button != 1 || self.drag.is_some() {
            return false;
        }

        match self.tool {
            Tool::Place(kind) => {
                let id = self
                    .doc
                    .add(kind, layout::snap_f(mx), layout::snap_f(my));
                self.selection = Some(Selection::Node(id));
                (self.report)(format!("{} added", name_of(&kind)));
                self.set_tool(Tool::Select);
                self.refresh();
            }
            Tool::Select => {
                if let Some((start, on_wire)) = self.wire_start(mx, my, mods.alt) {
                    self.drag = Some(Drag::Wiring {
                        start,
                        on_wire,
                        origin: (mx, my),
                        cur: start,
                        first_horizontal: None,
                        dragged: false,
                    });
                } else if let (true, Some(w)) = (mods.alt, self.wire_at(mx, my)) {
                    self.selection = Some(Selection::Wire(w));
                    self.drag = Some(Drag::MovingWire {
                        wire: w,
                        origin: (mx, my),
                        base: self.doc.clone(),
                        moved: false,
                    });
                } else if let Some(id) = self.node_at(mx, my) {
                    let n = self.doc.node(id);
                    let offset = (mx - n.x as f64, my - n.y as f64);
                    self.selection = Some(Selection::Node(id));
                    self.drag = Some(Drag::Moving {
                        node: id,
                        offset,
                        origin: (mx, my),
                        base: self.doc.clone(),
                        moved: false,
                    });
                } else {
                    self.selection = None;
                }
            }
        }
        true
    }

    fn motion(&mut self, mx: f64, my: f64, mods: Mods) -> bool {
        match self.drag.take() {
            None => {
                let hint = self.wire_start(mx, my, mods.alt).map(|(p, _)| p);
                let changed = hint != self.hint;
                self.hint = hint;
                changed
            }
            Some(Drag::Wiring {
                start,
                on_wire,
                origin,
                mut cur,
                mut first_horizontal,
                mut dragged,
            }) => {
                if !dragged && dist(origin, (mx, my)) >= DRAG_THRESHOLD {
                    dragged = true;
                }
                if dragged {
                    cur = (layout::snap_f(mx), layout::snap_f(my));
                    if cur == start {
                        first_horizontal = None;
                    } else if first_horizontal.is_none() {
                        // The first leg of the L follows the initial movement.
                        first_horizontal =
                            Some((mx - origin.0).abs() >= (my - origin.1).abs());
                    }
                }
                self.drag = Some(Drag::Wiring {
                    start,
                    on_wire,
                    origin,
                    cur,
                    first_horizontal,
                    dragged,
                });
                true
            }
            Some(Drag::Moving {
                node,
                offset,
                origin,
                base,
                mut moved,
            }) => {
                if moved || dist(origin, (mx, my)) >= DRAG_THRESHOLD {
                    moved = true;
                    let mut next = base.clone();
                    next.move_node(
                        node,
                        layout::snap_f(mx - offset.0),
                        layout::snap_f(my - offset.1),
                        !mods.shift,
                    );
                    self.doc = next;
                    self.refresh();
                }
                self.drag = Some(Drag::Moving {
                    node,
                    offset,
                    origin,
                    base,
                    moved,
                });
                true
            }
            Some(Drag::MovingWire {
                wire,
                origin,
                base,
                mut moved,
            }) => {
                if moved || dist(origin, (mx, my)) >= DRAG_THRESHOLD {
                    moved = true;
                    let mut next = base.clone();
                    let now = next.move_wire(
                        &wire,
                        layout::snap_f(mx - origin.0),
                        layout::snap_f(my - origin.1),
                    );
                    self.doc = next;
                    self.refresh();
                    self.selection = now.map(Selection::Wire);
                }
                self.drag = Some(Drag::MovingWire {
                    wire,
                    origin,
                    base,
                    moved,
                });
                true
            }
        }
    }

    fn release(&mut self) -> bool {
        match self.drag.take() {
            Some(Drag::Wiring {
                start,
                on_wire,
                cur,
                first_horizontal,
                dragged,
                ..
            }) => {
                if dragged && cur != start {
                    let shortened = self
                        .doc
                        .draw_wire(start, cur, first_horizontal.unwrap_or(true));
                    let msg = if shortened { "wire shortened" } else { "wire added" };
                    (self.report)(msg.to_string());
                    self.refresh();
                } else {
                    // Not dragged far enough: it was a click on the wire.
                    self.selection = on_wire.map(Selection::Wire);
                }
            }
            Some(Drag::Moving { node, moved, .. }) => {
                if moved {
                    (self.report)("node moved".to_string());
                } else if let Kind::Input = self.doc.node(node).kind {
                    self.doc.toggle(node);
                    (self.report)("input toggled".to_string());
                    self.refresh();
                }
            }
            Some(Drag::MovingWire { moved, .. }) => {
                if moved {
                    (self.report)("wire moved".to_string());
                }
            }
            None => {}
        }
        true
    }

    fn key(&mut self, key: gdk::keys::Key) -> bool {
        use gdk::keys::constants::{BackSpace, Delete, Escape};
        if key == Delete || key == BackSpace {
            self.delete_selection();
            true
        } else if key == Escape {
            match self.drag.take() {
                Some(Drag::Moving { base, .. }) | Some(Drag::MovingWire { base, .. }) => {
                    self.doc = base;
                    self.refresh();
                }
                _ => {}
            }
            self.set_tool(Tool::Select);
            true
        } else {
            false
        }
    }
}

fn mods_of(state: gdk::ModifierType) -> Mods {
    Mods {
        shift: state.contains(gdk::ModifierType::SHIFT_MASK),
        alt: state.contains(gdk::ModifierType::MOD1_MASK),
    }
}

pub struct Canvas {
    pub widget: gtk::DrawingArea,
    state: Rc<RefCell<CanvasState>>,
}

impl Canvas {
    pub fn new() -> Self {
        let state = Rc::new(RefCell::new(CanvasState::new()));
        let area = gtk::DrawingArea::new();
        area.set_can_focus(true);
        area.add_events(
            gdk::EventMask::EXPOSURE_MASK
                | gdk::EventMask::BUTTON_PRESS_MASK
                | gdk::EventMask::BUTTON_RELEASE_MASK
                | gdk::EventMask::POINTER_MOTION_MASK
                | gdk::EventMask::KEY_PRESS_MASK,
        );

        {
            let state = state.clone();
            area.connect_draw(move |area, cr| {
                let st = state.borrow();
                let alloc = area.allocation();
                let view = st.view();
                render(
                    cr,
                    &st.doc,
                    alloc.width() as f64,
                    alloc.height() as f64,
                    &view,
                );
                glib::Propagation::Stop
            });
        }
        {
            let state = state.clone();
            area.connect_button_press_event(move |area, ev| {
                area.grab_focus();
                let mut st = state.borrow_mut();
                let (x, y) = ev.position();
                st.press(ev.button(), mods_of(ev.state()), x, y);
                area.queue_draw();
                glib::Propagation::Proceed
            });
        }
        {
            let state = state.clone();
            area.connect_button_release_event(move |area, _ev| {
                let mut st = state.borrow_mut();
                st.release();
                area.queue_draw();
                glib::Propagation::Proceed
            });
        }
        {
            let state = state.clone();
            area.connect_motion_notify_event(move |area, ev| {
                let mut st = state.borrow_mut();
                let (x, y) = ev.position();
                if st.motion(x, y, mods_of(ev.state())) {
                    area.queue_draw();
                }
                glib::Propagation::Proceed
            });
        }
        {
            let state = state.clone();
            area.connect_key_press_event(move |area, ev| {
                let mut st = state.borrow_mut();
                if st.key(ev.keyval()) {
                    area.queue_draw();
                    glib::Propagation::Proceed
                } else {
                    glib::Propagation::Stop
                }
            });
        }

        Canvas { widget: area, state }
    }

    pub fn board(&self) -> std::cell::Ref<'_, Doc> {
        std::cell::Ref::map(self.state.borrow(), |s| &s.doc)
    }

    pub fn build_circuit(&self) -> Circuit {
        self.state.borrow().doc.build()
    }

    pub fn set_status<F>(&self, f: F)
    where
        F: FnMut(String) + 'static,
    {
        self.state.borrow_mut().report = Box::new(f);
    }

    pub fn set_tool_callback<F>(&self, f: F)
    where
        F: FnMut(Tool) + 'static,
    {
        self.state.borrow_mut().tool_used = Box::new(f);
    }

    pub fn set_tool(&self, t: Tool) {
        self.state.borrow_mut().set_tool(t);
    }

    pub fn delete_selection(&self) {
        self.state.borrow_mut().delete_selection();
    }

    pub fn clear(&self) {
        self.state.borrow_mut().clear();
    }

    pub fn handle(&self) -> CanvasHandle {
        CanvasHandle {
            state: self.state.clone(),
            widget: self.widget.clone(),
        }
    }
}

#[derive(Clone)]
pub struct CanvasHandle {
    state: Rc<RefCell<CanvasState>>,
    widget: gtk::DrawingArea,
}

impl CanvasHandle {
    pub fn build_circuit(&self) -> Circuit {
        self.state.borrow().doc.build()
    }

    pub fn set_tool(&self, t: Tool) {
        self.state.borrow_mut().set_tool(t);
    }

    pub fn set_status<F>(&self, f: F)
    where
        F: FnMut(String) + 'static,
    {
        self.state.borrow_mut().report = Box::new(f);
    }

    pub fn set_tool_callback<F>(&self, f: F)
    where
        F: FnMut(Tool) + 'static,
    {
        self.state.borrow_mut().tool_used = Box::new(f);
    }

    pub fn delete_selection(&self) {
        self.state.borrow_mut().delete_selection();
        self.widget.queue_draw();
    }

    pub fn clear(&self) {
        self.state.borrow_mut().clear();
        self.widget.queue_draw();
    }

    pub fn queue_draw(&self) {
        self.widget.queue_draw();
    }
}
