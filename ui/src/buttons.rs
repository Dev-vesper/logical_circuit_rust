//! The tool bar: one radio button per tool, plus the actions.
//!
//! `select_handle` returns a closure the canvas can call when it disarms
//! a tool by itself (Escape, or placing a node). The `updating` flag stops
//! the button's `toggled` signal from calling back into the canvas while
//! that programmatic activation runs — otherwise the `RefCell<CanvasState>`
//! would be re-borrowed inside `set_tool`.

use std::cell::Cell;
use std::rc::Rc;

use gtk::prelude::*;
use logic::Kind;

use crate::canvas::{CanvasHandle, Tool};

pub struct Buttons {
    pub box_widget: gtk::Box,
    buttons: Vec<(Tool, gtk::RadioButton)>,
    updating: Rc<Cell<bool>>,
}

pub fn create(canvas: &CanvasHandle, on_truth_table: impl Fn() + 'static) -> Buttons {
    let vbox = gtk::Box::new(gtk::Orientation::Horizontal, 6);
    vbox.set_border_width(6);

    let mut buttons: Vec<(Tool, gtk::RadioButton)> = Vec::new();
    let mut group: Option<gtk::RadioButton> = None;
    let updating = Rc::new(Cell::new(false));

    let mut arm = |tool: Tool, label: &str| {
        let button = gtk::RadioButton::with_label(label);
        if let Some(g) = &group {
            button.join_group(Some(g));
        } else {
            group = Some(button.clone());
        }
        vbox.pack_start(&button, false, false, 0);

        let canvas = canvas.clone();
        let updating = updating.clone();
        button.connect_toggled(move |b| {
            if b.is_active() && !updating.get() {
                canvas.set_tool(tool);
            }
        });
        buttons.push((tool, button));
    };

    arm(Tool::Select, "Select");
    arm(Tool::Place(Kind::Gate(logic::Gate::And)), "AND");
    arm(Tool::Place(Kind::Gate(logic::Gate::Or)), "OR");
    arm(Tool::Place(Kind::Gate(logic::Gate::Not)), "NOT");
    arm(Tool::Place(Kind::Input), "Input");

    // Select is armed first
    if let Some((_, b)) = buttons.iter().find(|(t, _)| *t == Tool::Select) {
        b.set_active(true);
    }

    let delete = gtk::Button::with_label("Delete");
    vbox.pack_start(&delete, false, false, 0);
    let c = canvas.clone();
    delete.connect_clicked(move |_| c.delete_selection());

    let clear = gtk::Button::with_label("Clear");
    vbox.pack_start(&clear, false, false, 0);
    let c = canvas.clone();
    clear.connect_clicked(move |_| c.clear());

    let truth = gtk::Button::with_label("Truth table");
    vbox.pack_start(&truth, false, false, 0);
    truth.connect_clicked(move |_| on_truth_table());

    Buttons {
        box_widget: vbox,
        buttons,
        updating,
    }
}

impl Buttons {
    pub fn widget(&self) -> &gtk::Box {
        &self.box_widget
    }

    /// A closure the canvas can call when its armed tool changes on its own.
    pub fn select_handle(&self) -> impl Fn(Tool) + 'static {
        let buttons = self.buttons.clone();
        let updating = self.updating.clone();
        move |tool| {
            if let Some((_, b)) = buttons.iter().find(|(t, _)| *t == tool) {
                updating.set(true);
                b.set_active(true);
                updating.set(false);
            }
        }
    }
}
