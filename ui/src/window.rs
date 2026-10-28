use std::rc::Rc;

use gtk::prelude::*;

use crate::buttons;
use crate::canvas::Canvas;
use crate::menu;
use crate::truth_panel::TruthPanel;

const KEYS_TEXT: &str = "\
Left click           arm a tool, then place it; otherwise select, or start\n\
                     dragging the node under the cursor\n\
Drag from a port     draw a wire, released on a free input port\n\
Click on an input    toggle it between 0 and 1\n\
Click on a wire      select it\n\
Click on a bend      drag the corner; on a segment, slide it\n\
Ctrl+click segment   insert a bend at that point\n\
Ctrl+click a bend    remove that bend\n\
Right click          delete the node or the wire under the cursor\n\
Delete, Backspace    delete the selection\n\
Escape               drop the drag and go back to Select\n\
\n\
Green cable = 1, grey = 0, amber = no value yet, red = feedback cycle.";

pub fn create() -> gtk::Window {
    let window = gtk::Window::new(gtk::WindowType::Toplevel);
    window.set_title("Logical circuit");
    window.set_default_size(960, 680);

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

    // Toolbar
    let toolbar = buttons::create(&handle, show_truth);
    vbox.pack_start(toolbar.widget(), false, false, 0);
    vbox.pack_start(&canvas.widget, true, true, 0);

    // Status bar
    let status = gtk::Label::new(Some("place a gate or an input to begin"));
    status.set_xalign(0.0);
    vbox.pack_start(&status, false, false, 0);

    {
        let status = status.clone();
        handle.set_status(move |m| status.set_text(&m));
    }

    // Keep the toolbar in sync when the canvas disarms a tool on its own.
    handle.set_tool_callback(toolbar.select_handle());

    // Re-draw whenever a canvas method changes state.
    // (queue_draw is already called by the canvas gesture handlers;
    //  the handle methods queue too, so we don't need a signal here.)

    window.connect_destroy(|_| gtk::main_quit());
    window.show_all();
    window
}
