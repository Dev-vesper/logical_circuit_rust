use std::rc::Rc;

use gtk::prelude::*;

use crate::canvas::Canvas;
use crate::menu;
use crate::toolbar;
use crate::truth_panel::TruthPanel;

const KEYS_TEXT: &str = "\
Drag from a pin, a wire end or a point on a wire   draw a wire (a diagonal drag makes an L)\n\
Drag back over a wire from its end                 shorten or delete that wire (dark preview)\n\
Click a wire                                       select it\n\
Alt+drag on the middle of a wire                   move the wire\n\
Alt+drag on empty space                            start a wire anywhere\n\
Drag a gate or input                               move it; its wires follow\n\
Shift while moving                                 leave the wires behind\n\
Escape                                             cancel a move / disarm the tool\n\
Right click                                        cancel a wire being drawn, or delete under the cursor\n\
Delete, Backspace                                  delete the selection\n\
Click on an input                                  toggle it between 0 and 1\n\
\n\
Wires never overlap: a pin or a wire end on a wire splits it, and a dot marks\n\
every point where more than two things meet.\n\
\n\
Dark green = 0, bright green = 1, brown = unknown/floating, warm grey = not\n\
attached to anything, red = conflicting outputs or a feedback cycle.";

pub fn create() -> gtk::Window {
    let window = gtk::Window::new(gtk::WindowType::Toplevel);
    window.set_title("Logical circuit");
    window.set_default_size(1080, 720);

    let vbox = gtk::Box::new(gtk::Orientation::Vertical, 0);
    window.add(&vbox);

    let canvas = Canvas::new();
    let handle = canvas.handle();
    let truth = Rc::new(TruthPanel::new());

    let show_truth = {
        let handle = handle.clone();
        let truth = truth.clone();
        move || truth.show(handle.build_circuit())
    };

    // Menu
    let win_for_quit = window.clone();
    let show_truth_menu = show_truth.clone();
    let menubar = menu::bar(
        move || unsafe { win_for_quit.destroy() },
        show_truth_menu,
        {
            let parent = window.clone();
            move || {
                let dialog = gtk::MessageDialog::new(
                    Some(&parent),
                    gtk::DialogFlags::MODAL,
                    gtk::MessageType::Info,
                    gtk::ButtonsType::Ok,
                    "Keys",
                );
                dialog.set_secondary_text(Some(KEYS_TEXT));
                dialog.run();
                unsafe {
                    dialog.destroy();
                }
            }
        },
    );
    vbox.pack_start(&menubar, false, false, 0);

    // Horizontal paned: sidebar on the left, canvas on the right.
    let paned = gtk::Paned::new(gtk::Orientation::Horizontal);
    paned.set_wide_handle(true);

    let toolbar = toolbar::create(&handle, show_truth.clone());
    paned.pack1(toolbar.widget(), false, false);
    paned.set_position(200);

    paned.pack2(&canvas.widget, true, true);
    vbox.pack_start(&paned, true, true, 0);

    // Status bar
    let status = gtk::Label::new(Some("place a gate or an input to begin"));
    status.set_xalign(0.0);
    status.set_margin_start(6);
    status.set_margin_end(6);
    status.set_margin_top(4);
    status.set_margin_bottom(4);
    vbox.pack_start(&status, false, false, 0);

    {
        let status = status.clone();
        handle.set_status(move |m| status.set_text(&m));
    }

    handle.set_tool_callback(toolbar.select_handle());

    window.connect_destroy(|_| gtk::main_quit());
    window.show_all();
    window
}
