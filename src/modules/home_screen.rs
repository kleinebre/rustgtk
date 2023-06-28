extern crate gtk;
use crate::modules::virtual_keyboard;
use gtk::prelude::*;
use gtk::{Button, Label};
use std::sync::Arc;
use std::sync::Mutex;
use virtual_keyboard::VirtualKeyboard;

pub struct SharedData {
    pub home_screen: Mutex<Option<HomeScreen>>,
    pub virtual_keyboard: Option<VirtualKeyboard>,
}
impl SharedData {
    pub fn new() -> SharedData {
        SharedData {
            home_screen: Mutex::new(None),
            virtual_keyboard: None,
        }
    }
}

pub struct HomeScreen {
    pub widget: gtk::Box,
}

impl HomeScreen {
    fn show(&self) {
        self.widget.show_all();
    }
    fn hide(&self) {
        self.widget.hide();
    }
    fn process_keyboard_reply(
        shared_data: &std::sync::MutexGuard<SharedData>, // already locked!
        returnbutton: virtual_keyboard::DialogResult,
    ) {
        let virtual_keyboard = &shared_data.virtual_keyboard.as_ref().unwrap();
        let binding = &shared_data.home_screen.lock().unwrap();
        let home_screen = binding.as_ref().unwrap();

        match returnbutton {
            virtual_keyboard::DialogResult::Ok => {
                println!(
                    "Keyboard click OK, val = {:?}",
                    virtual_keyboard.input.lock().as_ref().unwrap()
                );
            }
            _ => {
                println!("Dialog cancelled.");
            }
        }
        home_screen.show();
        virtual_keyboard.reset_input();
    }

    fn button_callback(button: &gtk::Button, shared_data: &Arc<Mutex<SharedData>>) {
        let button_label = button.label();
        match button_label {
            None => {
                // perhaps there's some other way to identify which button this is
                return;
            }
            Some(label) => {
                if label == "Keyboard" {
                    let binding = shared_data.lock().expect("poison");
                    let home_screen = binding.home_screen.lock().unwrap();
                    home_screen.as_ref().unwrap().hide();
                    let virtual_keyboard = binding.virtual_keyboard.as_ref().expect("not set");
                    virtual_keyboard.reset_input();
                    virtual_keyboard.show(
                        move |shared, returnbutton: virtual_keyboard::DialogResult| {
                            Self::process_keyboard_reply(shared, returnbutton);
                        },
                    );
                }
            }
        }
    }

    fn _create_widget(shared_data: Arc<Mutex<SharedData>>) -> gtk::Box {
        let shared_callback = move |button: &gtk::Button| {
            let _ = Self::button_callback(button, &shared_data);
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
    pub fn new(shared_data: Arc<Mutex<SharedData>>) -> HomeScreen {
        let widget = HomeScreen::_create_widget(Arc::clone(&shared_data));
        let instance = HomeScreen { widget };
        instance
    }
}
