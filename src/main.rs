extern crate gtk;
use glib;
use gtk::prelude::*;
use gtk::{Button, CssProvider, Label, Window, WindowType};
use std::sync::Arc;
use std::sync::Mutex;

pub const SCREEN_WIDTH: i32 = 800;
pub const SCREEN_HEIGHT: i32 = 480;

#[derive(Debug)]
enum DialogResult {
    Ok,
    Cancel,
}
struct SharedData {
    home_screen: Option<gtk::Box>,
    virtual_keyboard: Option<gtk::Box>,
    virtual_keyboard_input: String,
    virtual_keyboard_accept: String,
    virtual_keyboard_close_action: fn(&Arc<Mutex<SharedData>>, DialogResult),
}
impl SharedData {
    fn new() -> SharedData {
        SharedData {
            home_screen: None,
            virtual_keyboard: None,
            virtual_keyboard_input: "".to_string(),
            virtual_keyboard_accept: "".to_string(), // empty = accept all
            virtual_keyboard_close_action: |_, _| {},
        }
    }
}

struct HomeScreen {}
impl HomeScreen {
    fn show(shared_data: &Arc<Mutex<SharedData>>) {
        shared_data
            .lock()
            .expect("mutex poisoned")
            .home_screen
            .as_ref()
            .unwrap()
            .show_all();
    }
    fn hide(shared_data: &Arc<Mutex<SharedData>>) {
        shared_data
            .lock()
            .expect("mutex poisoned")
            .home_screen
            .as_ref()
            .unwrap()
            .hide();
    }
    fn create_widget(shared_data: Arc<Mutex<SharedData>>) -> gtk::Box {
        let shared_callback = move |button: &gtk::Button| {
            let button_label = button.label().unwrap();

            if button_label == "Keyboard" {
                shared_data
                    .lock()
                    .expect("mutex poisoned")
                    .virtual_keyboard_input = "".to_string();
                HomeScreen::hide(&shared_data);
                VirtualKeyboard::show(
                    &shared_data,
                    move |shared_data, returnbutton: DialogResult| {
                        match returnbutton {
                            DialogResult::Ok => {
                                let input = &shared_data
                                    .lock()
                                    .expect("mutex poisoned")
                                    .virtual_keyboard_input;
                                println!("Keyboard click OK, val = {:?}", input);
                            }
                            _ => {
                                println!("Dialog cancelled.");
                            }
                        }
                        HomeScreen::show(&shared_data);
                        shared_data
                            .lock()
                            .expect("mutex poisoned")
                            .virtual_keyboard_input = "".to_string();
                    },
                );
            }
        };

        let home_screen = gtk::Box::new(gtk::Orientation::Vertical, 5);

        let label = Label::new(Some("Hello, World!"));
        home_screen.pack_start(&label, true, true, 0);

        let button_opendia = Button::with_label("Keyboard");
        button_opendia.connect_clicked(shared_callback.clone());
        home_screen.pack_start(&button_opendia, true, true, 0);

        home_screen.show_all();
        home_screen
    }
}

struct VirtualKeyboard {}
impl VirtualKeyboard {
    fn append_input(shared_data: &Arc<Mutex<SharedData>>, label: &str) {
        let x: String = {
            // read the shared data, add a dot.
            format!(
                "{}{}",
                shared_data
                    .lock()
                    .expect("mutex poisoned")
                    .virtual_keyboard_input,
                label
            )
        };
        // write the dot back to shared data
        shared_data
            .lock()
            .expect("mutex poisoned")
            .virtual_keyboard_input = x;
    }
    fn show(
        shared_data: &Arc<Mutex<SharedData>>,
        close_action: fn(shared_data: &Arc<Mutex<SharedData>>, button: DialogResult),
    ) {
        {
            shared_data
                .lock()
                .expect("mutex poisoned")
                .virtual_keyboard_close_action = close_action;
        }
        {
            shared_data
                .lock()
                .expect("mutex poisoned")
                .virtual_keyboard
                .as_ref()
                .unwrap()
                .show_all();
        }
    }

    fn hide(shared_data: &Arc<Mutex<SharedData>>, close_button: DialogResult) {
        let action = {
            shared_data
                .lock()
                .expect("mutex poisoned")
                .virtual_keyboard_close_action
        };
        action(shared_data, close_button);
        {
            shared_data
                .lock()
                .expect("mutex poisoned")
                .virtual_keyboard_close_action = |_, _| {}
        };

        {
            shared_data
                .lock()
                .expect("mutex poisoned")
                .virtual_keyboard
                .as_ref()
                .unwrap()
                .hide();
        }
    }
    fn create_widget(shared_data: Arc<Mutex<SharedData>>) -> gtk::Box {
        let shared_callback = move |button: &gtk::Button| {
            let button_label = button.label().unwrap();

            if button_label == "OK" {
                VirtualKeyboard::hide(&shared_data, DialogResult::Ok);
                return;
            }
            if button_label == "Cancel" {
                VirtualKeyboard::hide(&shared_data, DialogResult::Cancel);
                return;
            }
            // any other button on the dialog
            VirtualKeyboard::append_input(&shared_data, &button_label);
        };

        let label = Label::new(Some("Enter something"));
        let button_a = Button::with_label("A");
        button_a.connect_clicked(shared_callback.clone());
        let button_b = Button::with_label("OK");
        button_b.connect_clicked(shared_callback.clone());
        let button_c = Button::with_label("Cancel");
        button_c.connect_clicked(shared_callback.clone());

        let virtual_keyboard = gtk::Box::new(gtk::Orientation::Vertical, 5);
        virtual_keyboard.pack_start(&label, true, true, 0);
        virtual_keyboard.pack_start(&button_a, true, true, 0);
        virtual_keyboard.pack_start(&button_b, true, true, 0);
        virtual_keyboard.pack_start(&button_c, true, true, 0);
        virtual_keyboard
    }
}

fn main() {
    gtk::init().expect("Failed to initialize GTK.");

    // define these before we move them into closures...
    let shared_data = Arc::new(Mutex::new(SharedData::new()));

    let shareddata_for_timer = Arc::clone(&shared_data);

    // timer thread
    let _source_id = glib::timeout_add_local(std::time::Duration::from_millis(100), move || {
        println!(
            "shared={}",
            shareddata_for_timer
                .lock()
                .expect("mutex poisoned")
                .virtual_keyboard_input
        );
        Continue(true)
    });

    let home_screen = HomeScreen::create_widget(Arc::clone(&shared_data));
    let virtual_keyboard = VirtualKeyboard::create_widget(Arc::clone(&shared_data));

    let vbox_main = gtk::Box::new(gtk::Orientation::Vertical, 5);
    vbox_main.pack_start(&home_screen, true, true, 0);
    vbox_main.pack_start(&virtual_keyboard, true, true, 0);

    // This allows showing/hiding the gui bits
    let shareddata_for_main = Arc::clone(&shared_data);
    {
        shareddata_for_main
            .lock()
            .expect("mutex poisoned")
            .home_screen = Some(home_screen);
        shareddata_for_main
            .lock()
            .expect("mutex poisoned")
            .virtual_keyboard = Some(virtual_keyboard);
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
    window1.set_default_size(SCREEN_WIDTH, SCREEN_HEIGHT);
    window1.connect_delete_event(|_, _| {
        gtk::main_quit();
        Inhibit(false)
    });

    window1.add(&vbox_main);
    vbox_main.show();
    window1.show();

    gtk::main();
}
