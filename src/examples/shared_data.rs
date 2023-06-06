extern crate gtk;
use gtk::{Button, Window, WindowType, Label, CssProvider};
use glib;
use gtk::prelude::*;
struct SharedData {
    last_clicked: String,
}
use std::sync::Arc;
use std::sync::Mutex;

fn main() {
    gtk::init().expect("Failed to initialize GTK.");

    let label = Label::new(Some("Hello, World!"));

    let sd: SharedData = SharedData { last_clicked: "START".to_string() };
    // define these before we move them into closures...
    let shareddata_for_button = Arc::new(Mutex::new(sd));
    let shareddata_for_timer = Arc::clone(&shareddata_for_button);

    // button with handler
    let button = Button::with_label("Click Me!");
    button.connect_clicked(move |_| {
        let x: String = {
            // read the shared data, add a dot.
            format!("{}.", shareddata_for_button.lock().expect("mutex poisoned").last_clicked)
        };
        // write the dot back to shared data
        shareddata_for_button.lock().expect("mutex poisoned").last_clicked = x;
    });

    // timer thread
    let _source_id = glib::timeout_add_local(std::time::Duration::from_millis(100), move || {
        println!("shared={}", shareddata_for_timer.lock().expect("mutex poisoned").last_clicked);
        Continue(true)
    });


    let vbox = gtk::Box::new(gtk::Orientation::Vertical, 5);
    vbox.pack_start(&label, true, true, 0);
    vbox.pack_start(&button, true, true, 0);

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

    // define the window
    let window = Window::new(WindowType::Toplevel);
    window.set_title("Hello, World!");
    window.set_default_size(250, 50);
    window.connect_delete_event(|_, _| {
        gtk::main_quit();
        Inhibit(false)
    });

    window.add(&vbox);
    window.show_all();

    gtk::main();
}
