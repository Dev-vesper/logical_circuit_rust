use cairo::Context;
use logic::{Gate, Kind, TruthTable, Value};
use ui::canvas;
use ui::doc::Doc;
use ui::shapes;

fn write(path: &str, width: i32, height: i32, draw: impl Fn(&Context)) {
    let surface =
        cairo::ImageSurface::create(cairo::Format::ARGB32, width, height).unwrap();
    let cr = Context::new(&surface).unwrap();
    shapes::set_color(&cr, shapes::PAPER);
    cr.paint();
    draw(&cr);
    surface.write_to_png(path).unwrap();
    println!("wrote {} ({}x{})", path, width, height);
}

fn main() {
    write("shapes.png", 280, 150, |cr| {
        shapes::draw_gate(cr, 20.0, 20.0, Gate::And);
        shapes::draw_gate(cr, 110.0, 20.0, Gate::Or);
        shapes::draw_gate(cr, 200.0, 20.0, Gate::Not);
        shapes::draw_input(cr, 20.0, 20.0, "A", Value::High);
    });

    let mut doc = Doc::new();
    let a = doc.add(Kind::Input, 40.0, 40.0);
    let b = doc.add(Kind::Input, 40.0, 140.0);
    let conj = doc.add(Kind::Gate(Gate::And), 220.0, 80.0);
    let inv = doc.add(Kind::Gate(Gate::Not), 380.0, 100.0);
    doc.toggle(a);
    doc.connect(a, conj, Some(0));
    doc.connect(b, conj, Some(1));
    doc.connect(conj, inv, None);
    write("board.png", 520, 260, |cr| {
        canvas::render_board(cr, &doc, 520.0, 260.0)
    });

    doc.set_route(
        inv,
        0,
        vec![
            (320.0, 110.0),
            (320.0, 200.0),
            (340.0, 200.0),
            (340.0, 130.0),
        ],
    );
    write("routed.png", 520, 260, |cr| {
        canvas::render_board(cr, &doc, 520.0, 260.0)
    });

    doc.move_to(inv, 380.0, 180.0);
    write("moved.png", 520, 260, |cr| {
        canvas::render_board(cr, &doc, 520.0, 260.0)
    });

    doc.move_to(conj, 240.0, 60.0);
    write("shifted.png", 520, 260, |cr| {
        canvas::render_board(cr, &doc, 520.0, 260.0)
    });

    let mut half = Doc::new();
    let a = half.add(Kind::Input, 40.0, 40.0);
    let conj = half.add(Kind::Gate(Gate::And), 220.0, 80.0);
    half.toggle(a);
    half.connect(a, conj, Some(0));
    write("partial.png", 520, 260, |cr| {
        canvas::render_board(cr, &half, 520.0, 260.0)
    });

    let circuit = doc.build();
    match TruthTable::of_circuit(&circuit, None) {
        Ok(t) => print!("{}", t.to_string_aligned()),
        Err(e) => println!("{}", e),
    }
}
