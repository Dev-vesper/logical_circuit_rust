use cairo::Context;
use logic::{Gate, Kind, TruthTable, Value};
use ui::canvas;
use ui::doc::Doc;
use ui::shapes;

fn write(path: &str, width: i32, height: i32, draw: impl Fn(&Context)) {
    let surface = cairo::ImageSurface::create(cairo::Format::ARgb32, width, height).unwrap();
    let cr = Context::new(&surface).unwrap();
    shapes::set_color(&cr, shapes::PAPER);
    let _ = cr.paint();
    draw(&cr);
    surface.write_to_png(&mut std::fs::File::create(path).unwrap()).unwrap();
    println!("wrote {} ({}x{})", path, width, height);
}

fn main() {
    write("shapes.png", 280, 150, |cr| {
        shapes::draw_gate(cr, 20.0, 20.0, Gate::And);
        shapes::draw_gate(cr, 110.0, 20.0, Gate::Or);
        shapes::draw_gate(cr, 200.0, 20.0, Gate::Not);
        shapes::draw_input(cr, 20.0, 80.0, "A", Value::High);
    });

    // A and B feed an AND, whose output goes through a NOT.
    // Pins: A out (90,70), B out (90,170), AND in (220,100) (220,120),
    //       AND out (280,110), NOT in (380,130).
    let mut doc = Doc::new();
    let a = doc.add(Kind::Input, 40, 40);
    let _b = doc.add(Kind::Input, 40, 140);
    let conj = doc.add(Kind::Gate(Gate::And), 220, 80);
    let inv = doc.add(Kind::Gate(Gate::Not), 380, 100);
    doc.toggle(a);

    // Each drag is one L (or one straight wire), like the Wiring tool.
    doc.draw_wire((90, 70), (190, 100), true);
    doc.draw_wire((190, 100), (220, 100), true);
    doc.draw_wire((90, 170), (190, 120), true);
    doc.draw_wire((190, 120), (220, 120), true);
    doc.draw_wire((280, 110), (330, 130), true);
    doc.draw_wire((330, 130), (380, 130), true);
    write("board.png", 520, 260, |cr| canvas::render_board(cr, &doc, 520.0, 260.0));

    // A wire end dropped in the middle of a wire splits it and joins them
    // (the junction dot appears).
    doc.draw_wire((190, 150), (150, 150), true);
    write("junction.png", 520, 260, |cr| canvas::render_board(cr, &doc, 520.0, 260.0));

    // Dragging back over a wire from its end shortens it.
    doc.draw_wire((380, 130), (350, 130), true);
    write("shortened.png", 520, 260, |cr| canvas::render_board(cr, &doc, 520.0, 260.0));
    doc.draw_wire((350, 130), (380, 130), true); // and drawing forward restores it

    // Moving a gate keeps its connections.
    doc.move_node(inv, 400, 100, true);
    write("moved.png", 520, 260, |cr| canvas::render_board(cr, &doc, 520.0, 260.0));

    // A gate with a pin left unwired shows blue (unknown) values.
    let mut half = Doc::new();
    let a = half.add(Kind::Input, 40, 40);
    let _conj = half.add(Kind::Gate(Gate::And), 220, 80);
    half.toggle(a);
    half.draw_wire((90, 70), (190, 100), true);
    half.draw_wire((190, 100), (220, 100), true);
    write("partial.png", 520, 260, |cr| canvas::render_board(cr, &half, 520.0, 260.0));

    let circuit = doc.build();
    match TruthTable::of_circuit(&circuit, None) {
        Ok(t) => print!("{}", t.to_string_aligned()),
        Err(e) => println!("{}", e),
    }
}
