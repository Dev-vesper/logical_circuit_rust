//! The left sidebar: a vertical tool palette with grouped tools and actions.
//!
//! The layout follows common diagram editors: a "Select" tool at the top,
//! then groups of component types (gates, inputs), and finally a row of
//! actions (Delete, Clear, Truth table). A separator and a section title
//! precede each group. `select_handle` works exactly as before: it returns
//! a closure the canvas can call to synchronise the active tool when the
//! canvas disarms it on its own (Escape, or placing a node).

use std::cell::Cell;
use std::rc::Rc;

use gtk::prelude::*;
use logic::Kind;

use crate::canvas::{CanvasHandle, Tool};

pub struct Toolbar {
    pub box_widget: gtk::Box,
    buttons: Vec<(Tool, gtk::RadioButton)>,
    updating: Rc<Cell<bool>>,
}

/// Helper: create a section title label.
fn section_label(text: &str) -> gtk::Label {
    let label = gtk::Label::new(Some(text));
    label.set_xalign(0.0);
    label.set_margin_top(12);
    label.set_margin_bottom(4);
    label.style_context().add_class("section-title");
    label
}

pub fn create(canvas: &CanvasHandle, on_truth_table: impl Fn() + 'static) -> Toolbar {
    // The sidebar itself.
    let sidebar = gtk::Box::new(gtk::Orientation::Vertical, 2);
    sidebar.set_border_width(8);
    sidebar.set_size_request(180, -1); // fixed width, full height
    sidebar.style_context().add_class("tool-sidebar");

    // A scrollable area so the sidebar can grow without breaking the layout.
    let scroller = gtk::ScrolledWindow::new(gtk::Adjustment::NONE, gtk::Adjustment::NONE);
    scroller.set_policy(gtk::PolicyType::Never, gtk::PolicyType::Automatic);
    scroller.add(&sidebar);

    // Wrap in an outer box to keep the scroller as the main child of the
    // window when packed.  The caller packs `box_widget` into the window.
    let outer = gtk::Box::new(gtk::Orientation::Vertical, 0);
    outer.pack_start(&scroller, true, true, 0);

    let mut buttons: Vec<(Tool, gtk::RadioButton)> = Vec::new();
    let mut group: Option<gtk::RadioButton> = None;
    let updating = Rc::new(Cell::new(false));

    // --- Select tool (top, always visible) ---
    sidebar.pack_start(&section_label("Tools"), false, false, 0);

    let mut arm = |parent: &gtk::Box,
                   tool: Tool,
                   label: &str,
                   buttons: &mut Vec<(Tool, gtk::RadioButton)>,
                   group: &mut Option<gtk::RadioButton>,
                   updating: &Rc<Cell<bool>>| {
        let button = gtk::RadioButton::with_label(label);
        if let Some(g) = group {
            button.join_group(Some(g));
        } else {
            *group = Some(button.clone());
        }
        button.set_margin_start(4);
        button.set_margin_end(4);
        button.set_margin_top(2);
        button.set_margin_bottom(2);
        button.style_context().add_class("tool-button");
        parent.pack_start(&button, false, false, 0);

        let canvas = canvas.clone();
        let updating = updating.clone();
        button.connect_toggled(move |b| {
            if b.is_active() && !updating.get() {
                canvas.set_tool(tool);
            }
        });
        buttons.push((tool, button));
    };

    arm(
        &sidebar,
        Tool::Select,
        "Select",
        &mut buttons,
        &mut group,
        &updating,
    );

    // --- Gates ---
    sidebar.pack_start(&gtk::Separator::new(gtk::Orientation::Horizontal), false, false, 0);
    sidebar.pack_start(&section_label("Gates"), false, false, 0);

    arm(
        &sidebar,
        Tool::Place(Kind::Gate(logic::Gate::And)),
        "AND",
        &mut buttons,
        &mut group,
        &updating,
    );
    arm(
        &sidebar,
        Tool::Place(Kind::Gate(logic::Gate::Or)),
        "OR",
        &mut buttons,
        &mut group,
        &updating,
    );
    arm(
        &sidebar,
        Tool::Place(Kind::Gate(logic::Gate::Not)),
        "NOT",
        &mut buttons,
        &mut group,
        &updating,
    );

    // --- Inputs ---
    sidebar.pack_start(&gtk::Separator::new(gtk::Orientation::Horizontal), false, false, 0);
    sidebar.pack_start(&section_label("Inputs"), false, false, 0);

    arm(
        &sidebar,
        Tool::Place(Kind::Input),
        "Input",
        &mut buttons,
        &mut group,
        &updating,
    );

    // --- Actions ---
    sidebar.pack_start(&gtk::Separator::new(gtk::Orientation::Horizontal), false, false, 0);
    sidebar.pack_start(&section_label("Actions"), false, false, 0);

    let delete = gtk::Button::with_label("Delete");
    delete.set_margin_start(4);
    delete.set_margin_end(4);
    delete.set_margin_top(2);
    delete.set_margin_bottom(2);
    delete.style_context().add_class("action-button");
    sidebar.pack_start(&delete, false, false, 0);
    let c = canvas.clone();
    delete.connect_clicked(move |_| c.delete_selection());

    let clear = gtk::Button::with_label("Clear");
    clear.set_margin_start(4);
    clear.set_margin_end(4);
    clear.set_margin_top(2);
    clear.set_margin_bottom(2);
    clear.style_context().add_class("action-button");
    sidebar.pack_start(&clear, false, false, 0);
    let c = canvas.clone();
    clear.connect_clicked(move |_| c.clear());

    let truth = gtk::Button::with_label("Truth table");
    truth.set_margin_start(4);
    truth.set_margin_end(4);
    truth.set_margin_top(2);
    truth.set_margin_bottom(2);
    truth.style_context().add_class("action-button");
    sidebar.pack_start(&truth, false, false, 0);
    truth.connect_clicked(move |_| on_truth_table());

    // Select is armed first
    if let Some((_, b)) = buttons.iter().find(|(t, _)| *t == Tool::Select) {
        b.set_active(true);
    }

    Toolbar {
        box_widget: outer,
        buttons,
        updating,
    }
}

impl Toolbar {
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
