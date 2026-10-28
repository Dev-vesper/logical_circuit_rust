use gtk::prelude::*;

pub fn bar<Q, T, K>(quit: Q, truth_table: T, keys: K) -> gtk::MenuBar
where
    Q: Fn() + 'static,
    T: Fn() + 'static,
    K: Fn() + 'static,
{
    let menubar = gtk::MenuBar::new();

    // File
    let file = gtk::MenuItem::with_label("File");
    let file_menu = gtk::Menu::new();
    let quit_item = gtk::MenuItem::with_label("Quit");
    quit_item.connect_activate(move |_| quit());
    file_menu.append(&quit_item);
    file.set_submenu(Some(&file_menu));
    menubar.append(&file);

    // View
    let view = gtk::MenuItem::with_label("View");
    let view_menu = gtk::Menu::new();
    let truth_item = gtk::MenuItem::with_label("Truth table");
    truth_item.connect_activate(move |_| truth_table());
    view_menu.append(&truth_item);
    view.set_submenu(Some(&view_menu));
    menubar.append(&view);

    // Help
    let help = gtk::MenuItem::with_label("Help");
    let help_menu = gtk::Menu::new();
    let keys_item = gtk::MenuItem::with_label("Keys");
    keys_item.connect_activate(move |_| keys());
    help_menu.append(&keys_item);
    help.set_submenu(Some(&help_menu));
    menubar.append(&help);

    menubar
}
