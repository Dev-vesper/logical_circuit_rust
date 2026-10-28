fn main() {
    if gtk::init().is_err() {
        eprintln!("Failed to initialise GTK");
        std::process::exit(1);
    }
    ui::style::apply();
    let _window = ui::window::create();
    gtk::main();
}
