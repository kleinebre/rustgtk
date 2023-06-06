extern crate gtk;
use glib;
use gtk::prelude::*;
use gtk::prelude::*;
use gtk::{Button, CssProvider, Label, Window, WindowType};
use gtk::{ButtonsType, Dialog, DialogFlags, MessageDialog, MessageType};
struct SharedData {
    last_clicked: String,
    window1: Option<Window>,
    window2: Option<Window>,
}
use std::sync::Arc;
use std::sync::Mutex;

fn main() {
    gtk::init().expect("Failed to initialize GTK.");

    let label = Label::new(Some("Hello, World!"));

    let mut sd: SharedData = SharedData {
        last_clicked: "".to_string(),
        window1: None,
        window2: None,
    };
    // define these before we move them into closures...
    let shareddata_for_button = Arc::new(Mutex::new(sd));
    let shareddata_for_timer = Arc::clone(&shareddata_for_button);
    let mut shareddata_for_main = Arc::clone(&shareddata_for_button);
    let mut shareddata_for_win1 = Arc::clone(&shareddata_for_button);
    let mut shareddata_for_win2 = Arc::clone(&shareddata_for_button);

    let shared_callback = move |button: &gtk::Button| {
        let label = button.label().unwrap();
        if button.parent().expect("").parent().as_ref().unwrap() == {
                shareddata_for_button
                .lock()
                .expect("mutex poisoned")
                .window1.as_ref().unwrap()
        } && label == "Input" {
            {
                shareddata_for_button
                .lock()
                .expect("mutex poisoned")
                .window1.as_ref().unwrap().hide();
            }
            {
            shareddata_for_button
                .lock()
                .expect("mutex poisoned")
                .window2.as_ref().unwrap().show_all();
            }
        }
        if label == "A" {
            {
                shareddata_for_button
                .lock()
                .expect("mutex poisoned")
                .window2.as_ref().unwrap().hide();
            }
            {
                        shareddata_for_button
                .lock()
                .expect("mutex poisoned")
                .window1.as_ref().unwrap().show_all();
            }
        }
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
    let button_opendia = Button::with_label("Input");
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

    let vbox1 = gtk::Box::new(gtk::Orientation::Vertical, 5);
    vbox1.pack_start(&label, true, true, 0);
    vbox1.pack_start(&button_opendia, true, true, 0);
    //vbox.pack_start(&button_b, true, true, 0);

    let vbox2 = gtk::Box::new(gtk::Orientation::Vertical, 5);
    vbox2.pack_start(&label, true, true, 0);
    vbox2.pack_start(&button_a, true, true, 0);
    vbox2.pack_start(&button_b, true, true, 0);

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

    window1.add(&vbox1);
    window1.show_all();
    {
        shareddata_for_main
            .lock()
            .expect("mutex poisoned")
            .window1 = Some(window1);
    }
    // define the window
    let window2 = Window::new(WindowType::Toplevel);
    window2.set_title("DIALOG");
    window2.set_default_size(250, 50);
    window2.connect_delete_event(move |a, b| {
            {
                shareddata_for_win2
                .lock()
                .expect("mutex poisoned")
                .window2.as_ref().unwrap().hide();
            }
            {
                shareddata_for_win2
                .lock()
                .expect("mutex poisoned")
                .window1.as_ref().unwrap().show_all();
            }
        println!("A={:?}, B={:?}", a, b);
        //a.hide();
        Inhibit(true)
    });
    window2.add(&vbox2);
    {
        shareddata_for_main
            .lock()
            .expect("mutex poisoned")
            .window2 = Some(window2);
    }

    //window2.show_all();

    gtk::main();
}
