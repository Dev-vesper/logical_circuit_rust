//! The board's truth table in a window of its own.

use gtk::prelude::*;
use logic::TruthTable;

pub struct TruthPanel {
    pub window: gtk::Window,
    pub text: gtk::Label,
}

impl TruthPanel {
    pub fn new() -> Self {
        let window = gtk::Window::new(gtk::WindowType::Toplevel);
        window.set_title("Truth table");
        window.set_default_size(380, 460);

        // ✕ only hides the window instead of destroying it, so the same
        // window can be shown again the next time `show` is called.
        // gtk-rs 0.18 has no `set_hide_on_close`, so we stop the
        // `delete-event` signal and hide the window ourselves.
        window.connect_delete_event(|w, _| {
            w.hide();
            glib::Propagation::Stop
        });

        let scroller = gtk::ScrolledWindow::new(gtk::Adjustment::NONE, gtk::Adjustment::NONE);
        scroller.set_policy(gtk::PolicyType::Automatic, gtk::PolicyType::Automatic);
        window.add(&scroller);

        let text = gtk::Label::new(None);
        text.set_xalign(0.0);
        text.set_yalign(0.0);
        text.set_selectable(true);
        text.style_context().add_class("truth-table");
        scroller.add(&text);

        TruthPanel { window, text }
    }

    pub fn show(&self, circuit: logic::Circuit) {
        let text = match TruthTable::of_circuit(&circuit, None) {
            Ok(t) => t.to_string_aligned(),
            Err(e) => format!("{}", e),
        };
        self.text.set_text(&text);
        // `deiconify` brings the window back from hidden/minimized,
        // `show_all` restores its children, `present` raises it.
        self.window.deiconify();
        self.window.show_all();
        self.window.present();
    }
}
