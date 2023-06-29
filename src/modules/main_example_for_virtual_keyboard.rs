extern crate gtk;

use rustgtk::modules::home_screen::{HomeScreen, SharedData};
use rustgtk::modules::virtual_keyboard;

use crate::virtual_keyboard::{VirtualKeyboard, SCREEN_HEIGHT, SCREEN_WIDTH};

use gtk::prelude::*;
use gtk::{Window, WindowType};

use std::sync::Arc;
use std::sync::Mutex;

fn main() {
    gtk::init().expect("Failed to initialize GTK.");
    // define the window
    let window = Window::new(WindowType::Toplevel);
    window.set_title("Hello, World!");
    window.set_default_size(SCREEN_WIDTH, SCREEN_HEIGHT);
    window.connect_delete_event(|_, _| {
        gtk::main_quit();
        Inhibit(false)
    });

    let vbox_main = gtk::Box::new(gtk::Orientation::Vertical, 5);
    window.add(&vbox_main);

    let style_context = vbox_main.style_context();
    style_context.add_class("root");

    let shared_data = Arc::new(Mutex::new(SharedData::new()));

    // Construct home screen
    let home_screen = HomeScreen::new(Arc::clone(&shared_data));
    vbox_main.pack_start(&home_screen.widget, true, true, 0);
    shared_data.lock().expect("poison").home_screen = Mutex::new(Some(home_screen));

    // Construct virtual keyboard instance
    let virtual_keyboard = VirtualKeyboard::new(
        &window,
        Arc::clone(&shared_data),
        "Please enter some text.",
        "", // empty=allow all chars (otherwise only allow listed chars)
    );
    vbox_main.pack_start(&virtual_keyboard.widget, true, true, 0);
    shared_data.lock().expect("poison").virtual_keyboard = Some(virtual_keyboard);

    vbox_main.show();
    window.show();

    gtk::main();
}
