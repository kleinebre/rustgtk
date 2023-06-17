extern crate gtk;
use glib;
use glib::object::Cast;
use gtk::prelude::*;
use gtk::prelude::*;

use gtk::{Button, CssProvider, Label, Window, WindowType};
// use gtk::{ButtonsType, Dialog, DialogFlags, MessageDialog, MessageType};
struct SharedData {
    last_clicked: String,
    box1: Option<gtk::Box>,
    box2: Option<gtk::Box>,
}
use std::sync::Arc;
use std::sync::Mutex;

fn main() {
    gtk::init().expect("Failed to initialize GTK.");

    let label = Label::new(Some("Hello, World!"));

    let mut sd: SharedData = SharedData {
        last_clicked: "".to_string(),
        box1: None,
        box2: None,
    };
    // define these before we move them into closures...
    let shareddata_for_button = Arc::new(Mutex::new(sd));
    let shareddata_for_timer = Arc::clone(&shareddata_for_button);
    let mut shareddata_for_main = Arc::clone(&shareddata_for_button);
    let mut shareddata_for_win1 = Arc::clone(&shareddata_for_button);
    let mut shareddata_for_win2 = Arc::clone(&shareddata_for_button);

    let shared_callback = move |button: &gtk::Button| {
        let label = button.label().unwrap();

        let dialog1: bool = {
            let lock = shareddata_for_button.lock().expect("mutex poisoned");
            button.parent().as_ref().unwrap() == lock.box1.as_ref().unwrap()
        };
        let dialog2: bool = {
            let lock = shareddata_for_button.lock().expect("mutex poisoned");
            button.parent().as_ref().unwrap() == lock.box2.as_ref().unwrap()
        };

        if dialog1 && label == "Keyboard" {
            {
                shareddata_for_button
                    .lock()
                    .expect("mutex poisoned")
                    .box1
                    .as_ref()
                    .unwrap()
                    .hide();
            }
            {
                shareddata_for_button
                    .lock()
                    .expect("mutex poisoned")
                    .box2
                    .as_ref()
                    .unwrap()
                    .show_all();
            }
        }
        if dialog2 && label == "A" {
            {
                shareddata_for_button
                    .lock()
                    .expect("mutex poisoned")
                    .box2
                    .as_ref()
                    .unwrap()
                    .hide();
            }
            {
                shareddata_for_button
                    .lock()
                    .expect("mutex poisoned")
                    .box1
                    .as_ref()
                    .unwrap()
                    .show_all();
            }
        }
        // any other button on any other dialog
        println!("Button {} was clicked", label);
        let x: String = {
            // read the shared data, add a dot.
            format!(
                "{}{}",
                shareddata_for_button
                    .lock()
                    .expect("mutex poisoned")
                    .last_clicked,
                label
            )
        };
        // write the dot back to shared data
        shareddata_for_button
            .lock()
            .expect("mutex poisoned")
            .last_clicked = x;
    };

    // button with handler
    let button_opendia = Button::with_label("Keyboard");
    button_opendia.connect_clicked(shared_callback.clone());

    let button_a = Button::with_label("A");
    button_a.connect_clicked(shared_callback.clone());
    let button_b = Button::with_label("B");
    button_b.connect_clicked(shared_callback.clone());

    // timer thread
    let _source_id = glib::timeout_add_local(std::time::Duration::from_millis(100), move || {
        println!(
            "shared={}",
            shareddata_for_timer
                .lock()
                .expect("mutex poisoned")
                .last_clicked
        );
        Continue(true)
    });
    let vbox_main = gtk::Box::new(gtk::Orientation::Vertical, 5);

    let box1 = gtk::Box::new(gtk::Orientation::Vertical, 5);
    box1.pack_start(&label, true, true, 0);
    box1.pack_start(&button_opendia, true, true, 0);
    box1.show_all();
    //vbox.pack_start(&button_b, true, true, 0);

    let box2 = gtk::Box::new(gtk::Orientation::Vertical, 5);
    box2.pack_start(&label, true, true, 0);
    box2.pack_start(&button_a, true, true, 0);
    box2.pack_start(&button_b, true, true, 0);
    vbox_main.pack_start(&box1, true, true, 0);
    vbox_main.pack_start(&box2, true, true, 0);
    vbox_main.show();
    {
        shareddata_for_main.lock().expect("mutex poisoned").box1 = Some(box1);
        shareddata_for_main.lock().expect("mutex poisoned").box2 = Some(box2);
    }

    // Create a CSS provider
    let css_provider = CssProvider::new();
    // Load the CSS data
    css_provider
        .load_from_data(
            "button { font-family: 'Arial'; font-size: 30px; font-weight: bold; }".as_bytes(),
        )
        .expect("Failed to load CSS");

    // Add the CSS provider to the default style context of the button
    gtk::StyleContext::add_provider_for_screen(
        &gdk::Screen::default().expect("Error initializing gtk css provider."),
        &css_provider,
        gtk::STYLE_PROVIDER_PRIORITY_USER,
    );

    // define the window
    let window1 = Window::new(WindowType::Toplevel);
    window1.set_title("Hello, World!");
    window1.set_default_size(250, 50);
    window1.connect_delete_event(|_, _| {
        gtk::main_quit();
        Inhibit(false)
    });

    window1.add(&vbox_main);
    window1.show();

    gtk::main();
}
