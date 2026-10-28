//! The board widget: dot grid, symbols, wires, live values, and the gestures.

use std::cell::RefCell;
use std::rc::Rc;

use gtk::prelude::*;
use logic::{Circuit, Kind, Value};

use crate::doc::{Doc, Id, Wire};
use crate::router;
use crate::shapes;

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

#[derive(Debug, Clone)]
pub enum Drag {
    Moving {
        node: Id,
        offset: (f64, f64),
        origin: (f64, f64),
    },
    Wiring {
        src: Id,
        /// نقاطی که کاربر با کلیک کامیت کرده، از پورت مبدأ شروع می‌شود.
        pts: Vec<(f64, f64)>,
        /// موقعیت فعلی نشانگر (یا موقعیت پورت مقصد اگر روی آن hover است).
        cursor: (f64, f64),
        /// پورت ورودی زیر نشانگر، اگر وجود دارد.
        target: Option<(Id, usize)>,
    },
    Adjusting {
        wire: Wire,
        segment: usize,
        vertical: bool,
        grab: f64,
        origin: (f64, f64),
        bends: Vec<(f64, f64)>,
    },
    Shaping {
        wire: Wire,
        index: usize,
        origin: (f64, f64),
        bends: Vec<(f64, f64)>,
    },
}

pub struct View<'a> {
    pub values: &'a [Value],
    pub on_cycle: &'a [bool],
    pub letters: &'a [String],
    pub selection: Option<Selection>,
    pub hover: Option<Wire>,
    pub drag: Option<&'a Drag>,
}

const SNAP_RADIUS: f64 = 14.0;
const WIRE_HIT_RADIUS: f64 = 8.0;
const HANDLE_RADIUS: f64 = 8.0;
const DOT_SPACING: f64 = shapes::CELL / 2.0;

fn dot_snap(v: f64) -> f64 {
    (v / DOT_SPACING).round() * DOT_SPACING
}

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
                    let mut y = 0.0;
                    while y < height {
                        let mut x = 0.0;
                        while x < width {
                            gcr.arc(x, y, 1.0, 0.0, 2.0 * std::f64::consts::PI);
                            shapes::set_color(&gcr, shapes::GRID);
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

// ---------- pure helpers over Doc ----------

fn snapshot(doc: &Doc) -> (Vec<Value>, Vec<bool>, Vec<String>) {
    let (_, values, cycle) = doc.evaluate();
    let count = doc.count();
    let mut on_cycle = vec![false; count];
    for &id in &cycle {
        if id < count {
            on_cycle[id] = true;
        }
    }
    let mut letters = vec![String::new(); count];
    let mut k = 0;
    doc.iter(|n| {
        if matches!(n.kind, Kind::Input) {
            letters[n.id] = letter(k);
            k += 1;
        }
    });
    (values, on_cycle, letters)
}

fn value_of(view: &View<'_>, id: Id) -> Value {
    if id < view.values.len() {
        view.values[id]
    } else {
        Value::Unknown
    }
}

fn on_cycle(view: &View<'_>, id: Id) -> bool {
    id < view.on_cycle.len() && view.on_cycle[id]
}

fn wire_ends(doc: &Doc, w: &Wire) -> (f64, f64, f64, f64) {
    let src = doc.node(w.src);
    let dst = doc.node(w.dst);
    let a = shapes::output_port(&src.kind);
    let b = shapes::input_ports(&dst.kind)[w.port];
    (src.x + a.x, src.y + a.y, dst.x + b.x, dst.y + b.y)
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

/// مسیر متعامد از `from` تا `to` با حداکثر یک گوشه.
/// اگر نقاط یکی باشند، خالی برمی‌گرداند.
fn orthogonal_path(from: (f64, f64), to: (f64, f64)) -> Vec<(f64, f64)> {
    if (from.0 - to.0).abs() < 0.5 && (from.1 - to.1).abs() < 0.5 {
        return vec![];
    }
    if (from.0 - to.0).abs() < 0.5 || (from.1 - to.1).abs() < 0.5 {
        return vec![to];
    }
    let dx = (to.0 - from.0).abs();
    let dy = (to.1 - from.1).abs();
    if dx >= dy {
        vec![(to.0, from.1), to]
    } else {
        vec![(from.0, to.1), to]
    }
}

fn normalize(bends: Vec<(f64, f64)>) -> Vec<(f64, f64)> {
    let mut out = Vec::new();
    let mut iter = bends.into_iter().peekable();
    while let Some(p) = iter.next() {
        if let Some(&q) = iter.peek() {
            if p == q {
                continue;
            }
        }
        out.push(p);
    }
    let mut result = Vec::new();
    let mut i = 0;
    while i < out.len() {
        if i + 2 < out.len() {
            let (x0, y0) = out[i];
            let (x1, y1) = out[i + 1];
            let (x2, y2) = out[i + 2];
            if (x0 == x1 && x1 == x2) || (y0 == y1 && y1 == y2) {
                result.push(out[i]);
                i += 2;
                continue;
            }
        }
        result.push(out[i]);
        i += 1;
    }
    result
}

fn orthogonalize(pts: Vec<(f64, f64)>) -> Vec<(f64, f64)> {
    if pts.len() < 2 {
        return pts;
    }
    let mut result = vec![pts[0]];
    let mut prev = pts[0];
    for i in 1..pts.len() {
        let p = pts[i];
        if p == prev {
            continue;
        }
        if prev.0 != p.0 && prev.1 != p.1 {
            let corner = if i + 1 < pts.len() && p.0 == pts[i + 1].0 {
                (p.0, prev.1)
            } else {
                (prev.0, p.1)
            };
            result.push(corner);
        }
        result.push(p);
        prev = p;
    }
    result
}

fn fit(doc: &Doc, w: &Wire, bends: Vec<(f64, f64)>) -> Vec<(f64, f64)> {
    let (sx, sy, dx, dy) = wire_ends(doc, w);
    let mut all = vec![(sx, sy)];
    all.extend(bends);
    all.push((dx, dy));
    let orth = orthogonalize(all);
    if orth.len() < 2 {
        return Vec::new();
    }
    orth[1..orth.len() - 1].to_vec()
}

fn with_bend(
    doc: &Doc,
    w: &Wire,
    bends: &[(f64, f64)],
    index: usize,
    x: f64,
    y: f64,
) -> Vec<(f64, f64)> {
    let moved: Vec<(f64, f64)> = bends
        .iter()
        .enumerate()
        .map(|(i, &p)| if i == index { (x, y) } else { p })
        .collect();
    fit(doc, w, moved)
}

fn obstacles_for(doc: &Doc, w: &Wire) -> Vec<(f64, f64, f64, f64)> {
    let mut out = Vec::new();
    doc.iter(|n| {
        if n.id != w.src && n.id != w.dst {
            out.push((n.x, n.y, shapes::SIZE, shapes::SIZE));
        }
    });
    out
}

fn bends_of(doc: &Doc, w: &Wire) -> Vec<(f64, f64)> {
    let stored = doc.route(w.dst, w.port);
    let bends = if stored.is_empty() {
        let (sx, sy, dx, dy) = wire_ends(doc, w);
        let obs = obstacles_for(doc, w);
        router::route((sx, sy), (dx, dy), &obs)
    } else {
        stored
    };
    fit(doc, w, bends)
}

fn wire_path(doc: &Doc, w: &Wire) -> Vec<(f64, f64)> {
    let (sx, sy, dx, dy) = wire_ends(doc, w);
    let mut path = vec![(sx, sy)];
    path.extend(bends_of(doc, w));
    path.push((dx, dy));
    path
}

fn draw_path(
    cr: &cairo::Context,
    color: (f64, f64, f64),
    width: f64,
    dash: Option<&[f64]>,
    pts: &[(f64, f64)],
) {
    if pts.is_empty() {
        return;
    }
    cr.move_to(pts[0].0, pts[0].1);
    for &(x, y) in &pts[1..] {
        cr.line_to(x, y);
    }
    shapes::set_color(cr, color);
    cr.set_line_width(width);
    cr.set_line_join(cairo::LineJoin::Round);
    if let Some(d) = dash {
        cr.set_dash(d, 0.0);
    }
    let _ = cr.stroke();
    cr.set_dash(&[], 0.0);
}

// ---------- rendering ----------

pub fn render(cr: &cairo::Context, doc: &Doc, width: f64, height: f64, view: &View<'_>) {
    paint_grid(cr, width, height);

    for w in doc.wires() {
        let picked = view.selection == Some(Selection::Wire(w));
        let hovered = view.hover == Some(w);
        let color = if on_cycle(view, w.src) && on_cycle(view, w.dst) {
            shapes::CYCLE
        } else if picked || hovered {
            shapes::SELECTED
        } else {
            shapes::color_of_value(value_of(view, w.src))
        };
        let width = if picked {
            3.5
        } else if hovered {
            3.0
        } else {
            2.5
        };
        draw_path(cr, color, width, None, &wire_path(doc, &w));
    }

    doc.iter(|n| {
        match n.kind {
            Kind::Input => shapes::draw_input(
                cr,
                n.x,
                n.y,
                &view.letters[n.id],
                if n.level { Value::High } else { Value::Low },
            ),
            Kind::Gate(g) => shapes::draw_gate(cr, n.x, n.y, g),
        }
        for (i, port) in shapes::input_ports(&n.kind).iter().enumerate() {
            let px = n.x + port.x;
            let py = n.y + port.y;
            if let Some(src) = n.ports[i] {
                cr.arc(px, py, shapes::PORT_RADIUS, 0.0, 2.0 * std::f64::consts::PI);
                shapes::set_color(cr, shapes::color_of_value(value_of(view, src)));
                cr.fill();
            }
            cr.arc(px, py, shapes::PORT_RADIUS, 0.0, 2.0 * std::f64::consts::PI);
            shapes::set_color(cr, shapes::INK);
            cr.set_line_width(1.5);
            cr.stroke();
        }
        let out = shapes::output_port(&n.kind);
        let ox = n.x + out.x;
        let oy = n.y + out.y;
        cr.arc(ox, oy, shapes::PORT_RADIUS, 0.0, 2.0 * std::f64::consts::PI);
        shapes::set_color(cr, shapes::color_of_value(value_of(view, n.id)));
        cr.fill();
        cr.arc(ox, oy, shapes::PORT_RADIUS, 0.0, 2.0 * std::f64::consts::PI);
        shapes::set_color(cr, shapes::INK);
        cr.set_line_width(1.5);
        cr.stroke();
        if view.selection == Some(Selection::Node(n.id)) {
            cr.rectangle(n.x - 6.0, n.y - 6.0, shapes::SIZE + 12.0, shapes::SIZE + 12.0);
            shapes::set_color(cr, shapes::SELECTED);
            cr.set_line_width(2.0);
            cr.set_dash(&[5.0, 3.0], 0.0);
            cr.stroke();
            cr.set_dash(&[], 0.0);
        }
    });

    let shown = match view.selection {
        Some(Selection::Wire(w)) => Some(w),
        _ => view.hover,
    };
    if let Some(w) = shown {
        for (x, y) in bends_of(doc, &w) {
            cr.rectangle(x - 3.0, y - 3.0, 6.0, 6.0);
            shapes::set_color(cr, shapes::PAPER);
            cr.fill_preserve();
            shapes::set_color(cr, shapes::SELECTED);
            cr.set_line_width(1.5);
            cr.stroke();
        }
    }

    if let Some(Drag::Wiring {
        pts,
        cursor,
        target,
        ..
    }) = view.drag
    {
        let mut path = pts.clone();
        let last = *pts.last().unwrap();
        let orth = orthogonal_path(last, *cursor);
        path.extend(orth);
        if path.len() < 2 {
            path.push(*cursor);
        }
        draw_path(cr, shapes::SELECTED, 2.5, Some(&[6.0, 4.0]), &path);
        match target {
            Some((dst, index)) => {
                let t = doc.node(*dst);
                let p = shapes::input_ports(&t.kind)[*index];
                cr.arc(t.x + p.x, t.y + p.y, 8.0, 0.0, 2.0 * std::f64::consts::PI);
                shapes::set_color(cr, shapes::SELECTED);
                cr.set_line_width(2.0);
                cr.stroke();
            }
            None => {
                cr.arc(cursor.0, cursor.1, 3.0, 0.0, 2.0 * std::f64::consts::PI);
                shapes::set_color(cr, shapes::SELECTED);
                cr.fill();
            }
        }
    }
}

pub fn render_board(cr: &cairo::Context, doc: &Doc, width: f64, height: f64) {
    let (values, on_cycle, letters) = snapshot(doc);
    let view = View {
        values: &values,
        on_cycle: &on_cycle,
        letters: &letters,
        selection: None,
        hover: None,
        drag: None,
    };
    render(cr, doc, width, height, &view);
}

// ---------- hit testing ----------

#[derive(Debug, Clone, Copy)]
enum PortHit {
    Out(Id),
    In(Id, usize),
}

// ---------- the widget ----------

struct CanvasState {
    doc: Doc,
    values: Vec<Value>,
    on_cycle: Vec<bool>,
    letters: Vec<String>,
    tool: Tool,
    selection: Option<Selection>,
    hover: Option<Wire>,
    drag: Option<Drag>,
    moved: bool,
    report: Box<dyn FnMut(String)>,
    tool_used: Box<dyn FnMut(Tool)>,
}

impl CanvasState {
    fn new() -> Self {
        CanvasState {
            doc: Doc::new(),
            values: Vec::new(),
            on_cycle: Vec::new(),
            letters: Vec::new(),
            tool: Tool::Select,
            selection: None,
            hover: None,
            drag: None,
            moved: false,
            report: Box::new(|_| {}),
            tool_used: Box::new(|_| {}),
        }
    }

    fn refresh(&mut self) {
        let (v, c, l) = snapshot(&self.doc);
        self.values = v;
        self.on_cycle = c;
        self.letters = l;
    }

    fn view(&self) -> View<'_> {
        View {
            values: &self.values,
            on_cycle: &self.on_cycle,
            letters: &self.letters,
            selection: self.selection,
            hover: self.hover,
            drag: self.drag.as_ref(),
        }
    }

    fn set_tool(&mut self, t: Tool) {
        if self.tool != t {
            self.tool = t;
            (self.tool_used)(t);
        }
    }

    fn node_at(&self, mx: f64, my: f64) -> Option<Id> {
        let mut hit = None;
        self.doc.iter(|n| {
            if mx >= n.x
                && mx <= n.x + shapes::SIZE
                && my >= n.y
                && my <= n.y + shapes::SIZE
            {
                hit = Some(n.id);
            }
        });
        hit
    }

    fn port_at(&self, mx: f64, my: f64) -> Option<PortHit> {
        let mut best: Option<(PortHit, f64)> = None;
        let mut consider = |x: f64, y: f64, h: PortHit| {
            let d = ((mx - x).powi(2) + (my - y).powi(2)).sqrt();
            if d <= SNAP_RADIUS && best.map_or(true, |(_, bd)| d < bd) {
                best = Some((h, d));
            }
        };
        self.doc.iter(|n| {
            let out = shapes::output_port(&n.kind);
            consider(n.x + out.x, n.y + out.y, PortHit::Out(n.id));
            for (i, port) in shapes::input_ports(&n.kind).iter().enumerate() {
                consider(n.x + port.x, n.y + port.y, PortHit::In(n.id, i));
            }
        });
        best.map(|(h, _)| h)
    }

    fn input_target(&self, mx: f64, my: f64) -> Option<((Id, usize), f64, f64)> {
        let mut best: Option<((Id, usize), f64, f64, f64)> = None;
        let mut consider = |x: f64, y: f64, h: (Id, usize)| {
            let d = ((mx - x).powi(2) + (my - y).powi(2)).sqrt();
            if d <= SNAP_RADIUS && best.map_or(true, |(_, _, _, bd)| d < bd) {
                best = Some((h, x, y, d));
            }
        };
        self.doc.iter(|n| {
            for (i, port) in shapes::input_ports(&n.kind).iter().enumerate() {
                consider(n.x + port.x, n.y + port.y, (n.id, i));
            }
        });
        best.map(|(h, x, y, _)| (h, x, y))
    }

    fn wire_target(&self, mx: f64, my: f64) -> (f64, f64, Option<(Id, usize)>) {
        match self.input_target(mx, my) {
            Some((h, x, y)) => (x, y, Some(h)),
            None => (dot_snap(mx), dot_snap(my), None),
        }
    }

    fn bend_at(&self, mx: f64, my: f64) -> Option<(Wire, usize)> {
        let mut best: Option<(Wire, usize, f64)> = None;
        for w in self.doc.wires() {
            for (i, (x, y)) in bends_of(&self.doc, &w).into_iter().enumerate() {
                let d = ((mx - x).powi(2) + (my - y).powi(2)).sqrt();
                if d > HANDLE_RADIUS {
                    continue;
                }
                if best.map_or(true, |(_, _, bd)| d < bd) {
                    best = Some((w, i, d));
                }
            }
        }
        best.map(|(w, i, _)| (w, i))
    }

    fn wire_at(&self, mx: f64, my: f64) -> Option<(Wire, usize)> {
        let mut best: Option<(Wire, usize, f64)> = None;
        for w in self.doc.wires() {
            let path = wire_path(&self.doc, &w);
            for i in 0..path.len().saturating_sub(1) {
                let (ax, ay) = path[i];
                let (bx, by) = path[i + 1];
                let d = distance_to_segment(mx, my, ax, ay, bx, by);
                if d > WIRE_HIT_RADIUS {
                    continue;
                }
                if best.map_or(true, |(_, _, bd)| d < bd) {
                    best = Some((w, i, d));
                }
            }
        }
        best.map(|(w, i, _)| (w, i))
    }

    fn segment_handle(&self, w: &Wire, segment: usize, mx: f64, my: f64) -> Option<(bool, f64)> {
        let bends = bends_of(&self.doc, w);
        let count = bends.len();
        let (sx, sy, dx, dy) = wire_ends(&self.doc, w);
        let point_at = |k: usize| -> (f64, f64) {
            if k == 0 {
                (sx, sy)
            } else if k == count + 1 {
                (dx, dy)
            } else {
                bends[k - 1]
            }
        };
        if segment > count {
            return None;
        }
        let (ax, ay) = point_at(segment);
        let (bx, _by) = point_at(segment + 1);
        let vertical = ax == bx;
        if !vertical && !(segment > 0 && segment < count) {
            return None;
        }
        Some((vertical, if vertical { mx - ax } else { my - ay }))
    }

    fn delete_selection(&mut self) {
        match self.selection {
            Some(Selection::Node(id)) => {
                self.doc.remove(id);
                (self.report)("node deleted".to_string());
            }
            Some(Selection::Wire(w)) => {
                self.doc.disconnect(w.dst, w.port);
                (self.report)("wire deleted".to_string());
            }
            None => (self.report)("nothing is selected".to_string()),
        }
        self.selection = None;
        self.hover = None;
        self.refresh();
    }

    fn clear(&mut self) {
        self.doc.clear();
        self.selection = None;
        self.hover = None;
        self.drag = None;
        (self.report)("the board is empty".to_string());
        self.refresh();
    }

    fn press(&mut self, button: u32, ctrl: bool, alt: bool, mx: f64, my: f64) -> bool {
        self.hover = None;

        // اگر در حال کشیدن سیم هستیم، هر کلیک چپ یک گوشه کامیت می‌کند.
        if button == 1 {
            if let Some(Drag::Wiring { pts, cursor, .. }) = &mut self.drag {
                let last = *pts.last().unwrap();
                let orth = orthogonal_path(last, *cursor);
                if !orth.is_empty() {
                    pts.extend(orth);
                }
                return true;
            }
            if self.drag.is_some() {
                return false;
            }
        }

        // Alt+کلیک روی سیم، آن را انتخاب می‌کند (قانون Logisim).
        if button == 1 && alt {
            if let Some((w, _)) = self.wire_at(mx, my) {
                self.selection = Some(Selection::Wire(w));
                return true;
            }
        }

        // Ctrl+کلیک روی گوشه حذف؛ روی سگمنت افزودن.
        if button == 1 && ctrl {
            if let Some((wire, index)) = self.bend_at(mx, my) {
                let mut bends = bends_of(&self.doc, &wire);
                if index < bends.len() {
                    bends.remove(index);
                    self.doc.set_route(wire.dst, wire.port, bends);
                    self.selection = Some(Selection::Wire(wire));
                    self.refresh();
                    (self.report)("bend removed".to_string());
                }
                return true;
            }
            if let Some((wire, seg)) = self.wire_at(mx, my) {
                if self.insert_bend(&wire, seg, mx, my) {
                    self.selection = Some(Selection::Wire(wire));
                    self.refresh();
                    (self.report)("bend added".to_string());
                }
                return true;
            }
            return true;
        }

        // راست‌کلیک وسط درگ سیم، آن را لغو می‌کند.
        if button == 3 && matches!(self.drag, Some(Drag::Wiring { .. })) {
            self.drag = None;
            return true;
        }

        match button {
            1 => match self.tool {
                Tool::Place(kind) => {
                    let id = self.doc.add(kind, shapes::snap(mx), shapes::snap(my));
                    self.selection = Some(Selection::Node(id));
                    (self.report)(format!("{} added", name_of(&kind)));
                    self.set_tool(Tool::Select);
                    self.refresh();
                }
                Tool::Select => match self.port_at(mx, my) {
                    Some(PortHit::Out(id)) => {
                        let node = self.doc.node(id);
                        let port = shapes::output_port(&node.kind);
                        let start = (node.x + port.x, node.y + port.y);
                        let (tx, ty, target) = self.wire_target(mx, my);
                        self.drag = Some(Drag::Wiring {
                            src: id,
                            pts: vec![start],
                            cursor: (tx, ty),
                            target,
                        });
                    }
                    Some(PortHit::In(dst, port)) => {
                        if self.doc.node(dst).ports[port].is_some() {
                            self.doc.disconnect(dst, port);
                            (self.report)("wire removed".to_string());
                            self.refresh();
                        }
                    }
                    None => {
                        if let Some(id) = self.node_at(mx, my) {
                            let node = self.doc.node(id);
                            self.drag = Some(Drag::Moving {
                                node: id,
                                offset: (mx - node.x, my - node.y),
                                origin: (mx, my),
                            });
                        } else if let Some((w, index)) = self.bend_at(mx, my) {
                            self.selection = Some(Selection::Wire(w));
                            self.drag = Some(Drag::Shaping {
                                wire: w,
                                index,
                                origin: (mx, my),
                                bends: bends_of(&self.doc, &w),
                            });
                        } else if let Some((w, segment)) = self.wire_at(mx, my) {
                            self.selection = Some(Selection::Wire(w));
                            if let Some((vertical, grab)) =
                                self.segment_handle(&w, segment, mx, my)
                            {
                                self.drag = Some(Drag::Adjusting {
                                    wire: w,
                                    segment,
                                    vertical,
                                    grab,
                                    origin: (mx, my),
                                    bends: bends_of(&self.doc, &w),
                                });
                            }
                        } else {
                            self.selection = None;
                        }
                    }
                },
            },
            3 => {
                if let Some(id) = self.node_at(mx, my) {
                    self.doc.remove(id);
                    self.selection = None;
                    (self.report)("node deleted".to_string());
                    self.refresh();
                } else if let Some((w, _)) = self.wire_at(mx, my) {
                    self.doc.disconnect(w.dst, w.port);
                    self.selection = None;
                    (self.report)("wire deleted".to_string());
                    self.refresh();
                }
            }
            _ => {}
        }
        true
    }

    fn insert_bend(&mut self, wire: &Wire, seg: usize, mx: f64, my: f64) -> bool {
        let mut bends = bends_of(&self.doc, wire);
        let path = wire_path(&self.doc, wire);
        if seg + 1 >= path.len() {
            return false;
        }
        let (ax, ay) = path[seg];
        let (bx, by) = path[seg + 1];
        let horizontal = (ay - by).abs() < 0.01;
        let vertical = (ax - bx).abs() < 0.01;
        let mut px = dot_snap(mx);
        let mut py = dot_snap(my);
        // If the snapped click lands exactly on the segment, nudge it one
        // dot perpendicular to the click side so the new corner is visible
        // (a collinear bend would be swallowed by `orthogonalize`).
        if horizontal && (py - ay).abs() < 0.01 {
            let dir = if my >= ay { 1.0 } else { -1.0 };
            py = ay + dir * DOT_SPACING;
        }
        if vertical && (px - ax).abs() < 0.01 {
            let dir = if mx >= ax { 1.0 } else { -1.0 };
            px = ax + dir * DOT_SPACING;
        }
        bends.insert(seg, (px, py));
        self.doc.set_route(wire.dst, wire.port, bends);
        true
    }

    fn motion(&mut self, mx: f64, my: f64) -> bool {
        let dragged_far = |ox: f64, oy: f64| ((mx - ox).powi(2) + (my - oy).powi(2)).sqrt() >= 4.0;
        let was_dragging = self.drag.is_some();

        let mut new_drag = None;
        if let Some(d) = self.drag.take() {
            match d {
                Drag::Moving { node, offset, origin } => {
                    self.doc
                        .move_to(node, shapes::snap(mx - offset.0), shapes::snap(my - offset.1));
                    if dragged_far(origin.0, origin.1) {
                        self.moved = true;
                    }
                    new_drag = Some(Drag::Moving { node, offset, origin });
                }
                Drag::Wiring { src, pts, .. } => {
                    let (tx, ty, tgt) = self.wire_target(mx, my);
                    new_drag = Some(Drag::Wiring {
                        src,
                        pts,
                        cursor: (tx, ty),
                        target: tgt,
                    });
                }
                Drag::Adjusting {
                    wire,
                    segment,
                    vertical,
                    grab,
                    origin,
                    mut bends,
                } => {
                    if dragged_far(origin.0, origin.1) {
                        let value = dot_snap(if vertical { mx - grab } else { my - grab });
                        for (i, b) in bends.iter_mut().enumerate() {
                            if i + 1 == segment || i == segment {
                                if vertical {
                                    b.0 = value;
                                } else {
                                    b.1 = value;
                                }
                            }
                        }
                        let refit = fit(&self.doc, &wire, bends.clone());
                        self.doc.set_route(wire.dst, wire.port, refit);
                        self.moved = true;
                    }
                    new_drag = Some(Drag::Adjusting {
                        wire,
                        segment,
                        vertical,
                        grab,
                        origin,
                        bends,
                    });
                }
                Drag::Shaping {
                    wire,
                    index,
                    origin,
                    bends,
                } => {
                    if dragged_far(origin.0, origin.1) {
                        let next = with_bend(
                            &self.doc,
                            &wire,
                            &bends,
                            index,
                            dot_snap(mx),
                            dot_snap(my),
                        );
                        self.doc.set_route(wire.dst, wire.port, next);
                        self.moved = true;
                    }
                    new_drag = Some(Drag::Shaping {
                        wire,
                        index,
                        origin,
                        bends,
                    });
                }
            }
        }
        self.drag = new_drag;

        if was_dragging || self.drag.is_some() {
            return true;
        }
        let under = self.wire_at(mx, my).map(|(w, _)| w);
        let changed = under != self.hover;
        self.hover = under;
        changed
    }

    fn release(&mut self, mx: f64, my: f64) -> bool {
        if let Some(d) = self.drag.take() {
            match d {
                Drag::Moving { node, .. } if !self.moved => {
                    if let Kind::Input = self.doc.node(node).kind {
                        self.doc.toggle(node);
                        (self.report)("input toggled".to_string());
                        self.refresh();
                    }
                }
                Drag::Wiring {
                    src,
                    mut pts,
                    cursor,
                    target,
                } => {
                    let last = *pts.last().unwrap();
                    let orth = orthogonal_path(last, cursor);
                    if !orth.is_empty() {
                        pts.extend(orth);
                    }
                    match target {
                        Some((dst, port)) => {
                            let waypoints: Vec<(f64, f64)> = if pts.len() >= 2 {
                                let mut wp = pts[1..].to_vec();
                                wp.pop();
                                normalize(wp)
                            } else {
                                vec![]
                            };
                            if self.doc.connect(src, dst, Some(port)).is_some() {
                                if !waypoints.is_empty() {
                                    self.doc.set_route(dst, port, waypoints);
                                }
                                (self.report)("wire added".to_string());
                                self.refresh();
                            } else {
                                (self.report)(
                                    "every input port of that gate is wired already".to_string(),
                                );
                            }
                        }
                        None => {
                            (self.report)("a wire has to end on an input port".to_string());
                        }
                    }
                }
                Drag::Adjusting { wire, bends, .. } => {
                    if self.moved {
                        let norm = normalize(fit(&self.doc, &wire, bends));
                        self.doc.set_route(wire.dst, wire.port, norm);
                        (self.report)("wire adjusted".to_string());
                    }
                }
                Drag::Shaping {
                    wire,
                    index,
                    bends,
                    ..
                } => {
                    if self.moved {
                        let next = normalize(with_bend(
                            &self.doc,
                            &wire,
                            &bends,
                            index,
                            dot_snap(mx),
                            dot_snap(my),
                        ));
                        self.doc.set_route(wire.dst, wire.port, next);
                        (self.report)("wire adjusted".to_string());
                    }
                }
                Drag::Moving { .. } => {}
            }
        }
        self.moved = false;
        true
    }

    fn key(&mut self, key: gdk::keys::Key) -> bool {
        use gdk::keys::constants::{BackSpace, Delete, Escape};
        if key == Delete || key == BackSpace {
            self.delete_selection();
            true
        } else if key == Escape {
            self.drag = None;
            self.set_tool(Tool::Select);
            true
        } else {
            false
        }
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
                let ctrl = ev.state().contains(gdk::ModifierType::CONTROL_MASK);
                let alt = ev.state().contains(gdk::ModifierType::MOD1_MASK);
                st.press(ev.button(), ctrl, alt, ev.position().0, ev.position().1);
                area.queue_draw();
                glib::Propagation::Proceed
            });
        }
        {
            let state = state.clone();
            area.connect_button_release_event(move |area, ev| {
                let mut st = state.borrow_mut();
                st.release(ev.position().0, ev.position().1);
                area.queue_draw();
                glib::Propagation::Proceed
            });
        }
        {
            let state = state.clone();
            area.connect_motion_notify_event(move |area, ev| {
                let mut st = state.borrow_mut();
                if st.motion(ev.position().0, ev.position().1) {
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
