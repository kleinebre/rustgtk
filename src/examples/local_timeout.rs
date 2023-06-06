extern crate gtk;
//use gtk::prelude::*;
use gtk::{Button, Window, WindowType, Label, CssProvider};
use glib;
use gtk::prelude::*;

fn main() {
    gtk::init().expect("Failed to initialize GTK.");

    let window = Window::new(WindowType::Toplevel);
    window.set_title("Hello, World!");
    window.set_default_size(250, 50);

    window.connect_delete_event(|_, _| {
        gtk::main_quit();
        Inhibit(false)
    });

    let button = Button::with_label("Click Me!");
    // Create a CSS provider
    let css_provider = CssProvider::new();
    // Load the CSS data
    css_provider.load_from_data("button { font-family: 'Arial'; font-size: 30px; font-weight: bold; }".as_bytes())
        .expect("Failed to load CSS");

    // Add the CSS provider to the default style context of the button
    gtk::StyleContext::add_provider_for_screen(
        &gdk::Screen::default().expect("Error initializing gtk css provider."),
        &css_provider,
        gtk::STYLE_PROVIDER_PRIORITY_USER,
    );

    button.connect_clicked(|_| {
        println!("Button clicked!");
    });

    let label = Label::new(Some("Hello, World!"));

    let vbox = gtk::Box::new(gtk::Orientation::Vertical, 5);
    vbox.pack_start(&label, true, true, 0);
    vbox.pack_start(&button, true, true, 0);

    window.add(&vbox);
    window.show_all();
    let _source_id = glib::timeout_add_local(std::time::Duration::from_millis(500), || {
        println!("Custom event triggered!");
        Continue(true)
    });

    gtk::main();
}
