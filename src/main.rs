extern crate gtk;
use rustgtk::modules::home_screen;
use rustgtk::modules::home_screen::{HomeScreen, SharedData};
use rustgtk::modules::virtual_keyboard;

use crate::virtual_keyboard::{
    VirtualKeyboard, ID_BACKSPACE, ID_CANCEL, ID_DELETE, ID_ENTER, ID_INSERT, ID_LEFT, ID_RIGHT,
    SCREEN_HEIGHT, SCREEN_WIDTH, VIRTUAL_KEYBOARD_CSS,
};

use gtk::prelude::*;
use gtk::{CssProvider, Window, WindowType};

use std::sync::Arc;
use std::sync::Mutex;

fn main() {
    gtk::init().expect("Failed to initialize GTK.");
    let vbox_main = gtk::Box::new(gtk::Orientation::Vertical, 5);
    let style_context = vbox_main.style_context();
    style_context.add_class("root");
    let shared_data = Arc::new(Mutex::new(SharedData::new()));

    let virtual_keyboard = VirtualKeyboard::new(
        Arc::clone(&shared_data),
        "Please enter some text.",
        "", // empty=allow all chars (otherwise only allow listed chars)
    );
    let home_screen = HomeScreen::new(Arc::clone(&shared_data));
    vbox_main.pack_start(&home_screen.widget, true, true, 0);
    vbox_main.pack_start(&virtual_keyboard.widget, true, true, 0);

    shared_data.lock().expect("poison").virtual_keyboard = Some(virtual_keyboard);
    shared_data.lock().expect("poison").home_screen = Mutex::new(Some(home_screen));

    /*
    let shareddata_for_timer = Arc::clone(&shared_data);

    // timer thread
    let _source_id = glib::timeout_add_local(std::time::Duration::from_millis(100), move || {
        let sd = &shareddata_for_timer.lock().expect("poison");
        let vk = sd.virtual_keyboard.as_ref();
        if vk.is_some() {
            println!("shared={}", vk.unwrap().input.lock().expect("poison"));
        }
        Continue(true)
    });
    */

    // Create a CSS provider
    let css_provider = CssProvider::new();
    // Load the CSS data
    css_provider
        .load_from_data(VIRTUAL_KEYBOARD_CSS.as_bytes())
        .expect("Failed to load CSS");

    // Add the CSS provider to the default style context of the button
    gtk::StyleContext::add_provider_for_screen(
        &gdk::Screen::default().expect("Error initializing gtk css provider."),
        &css_provider,
        gtk::STYLE_PROVIDER_PRIORITY_USER,
    );

    // define the window
    let window = Window::new(WindowType::Toplevel);
    let shareddata_for_keypress = Arc::clone(&shared_data);

    window.connect_local("key_press_event", false, move |values| {
        let sd = shareddata_for_keypress.lock().expect("poison");
        let vk = sd.virtual_keyboard.as_ref();
        if let Some(keyboard) = vk {
            println!("keypress");
            let raw_event = &values[1].get::<gdk::Event>().unwrap();
            // You have to cast to the correct event type to access some of the fields
            match raw_event.downcast_ref::<gdk::EventKey>() {
                Some(event) => {
                    let keyval: u32 = *event.keyval();
                    // let state: gdk::ModifierType = event.state();
                    let (plain_key, special_key) = if keyval <= 255 {
                        let character = char::from(keyval as u8);
                        let key = character.to_string();
                        if keyboard.accept != "" {
                            if !keyboard.accept.contains(&key) {
                                ("".to_string(), "".to_string())
                            } else {
                                (key, "".to_string())
                            }
                        } else {
                            (key, "".to_string())
                        }
                    } else {
                        if keyval == *gdk::keys::constants::BackSpace {
                            ("".to_string(), ID_BACKSPACE.to_string())
                        } else if keyval == *gdk::keys::constants::Delete {
                            ("".to_string(), ID_DELETE.to_string())
                        } else if keyval == *gdk::keys::constants::Insert {
                            ("".to_string(), ID_INSERT.to_string())
                        } else if keyval == *gdk::keys::constants::Left {
                            ("".to_string(), ID_LEFT.to_string())
                        } else if keyval == *gdk::keys::constants::Right {
                            ("".to_string(), ID_RIGHT.to_string())
                        } else if keyval == *gdk::keys::constants::Return {
                            ("".to_string(), ID_ENTER.to_string())
                        } else if keyval == *gdk::keys::constants::Escape {
                            ("".to_string(), ID_CANCEL.to_string())
                        } else {
                            ("".to_string(), "".to_string())
                        }
                    };
                    if format!("{}{}", plain_key, special_key) != "" {
                        keyboard.handle_key(&sd, &plain_key, &special_key);
                    }
                }
                None => {}
            }
        };
        Some(true.into())
    });

    window.set_title("Hello, World!");
    window.set_default_size(SCREEN_WIDTH, SCREEN_HEIGHT);
    window.connect_delete_event(|_, _| {
        gtk::main_quit();
        Inhibit(false)
    });

    window.add(&vbox_main);
    vbox_main.show();
    window.show();

    gtk::main();
}
