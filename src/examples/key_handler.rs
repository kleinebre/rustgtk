extern crate gtk;
use glib;
use gdk_sys;
use gtk::prelude::*;
use gtk::{Button, CssProvider, Label, Window, WindowType};
struct SharedData {
    last_clicked: String,
}
use std::sync::Arc;
use std::sync::Mutex;

fn main() {
    gtk::init().expect("Failed to initialize GTK.");

    let label = Label::new(Some("ESC with no modifiers to exit"));

    // button with handler
    let button_a = Button::with_label("A");

    let vbox = gtk::Box::new(gtk::Orientation::Vertical, 5);
    vbox.pack_start(&label, true, true, 0);
    vbox.pack_start(&button_a, true, true, 0);
    // define the window
    let window = Window::new(WindowType::Toplevel);
    let window_ref = &window;
    button_a.connect("key_press_event", false, move |values| {

        let raw_event = &values[1].get::<gdk::Event>().unwrap();
        // You have to cast to the correct event type to access some of the fields
        match raw_event.downcast_ref::<gdk::EventKey>() {
            Some(event) => {
                let keyval: u32 = *event.keyval();
                let state: gdk::ModifierType = event.state();
                if state == gdk::ModifierType::empty() {
                    if *event.keyval() == gdk_sys::GDK_KEY_Escape.try_into().unwrap() {
                        gtk::main_quit();
                    }
                }

                println!("key value: {:?}", keyval);
                println!("modifiers: {:?}", state);
             },
            None => {},
        }

        // I can't figure out how to actually set the value of result
        // Luckally returning false is good enough for now.
        Some(true.into())
    });

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
