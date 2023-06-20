extern crate gtk;
use glib;
use gtk::prelude::*;
use gtk::builders::LabelBuilder;
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
    home_screen: Mutex<Option<HomeScreen>>,
    virtual_keyboard: Option<VirtualKeyboard>,
}
impl SharedData {
    fn new() -> SharedData {
        SharedData {
            home_screen: Mutex::new(None),
            virtual_keyboard: None,
        }
    }
}

struct HomeScreen {
    widget: gtk::Box,
}

type DialogCloseAction = fn(&std::sync::MutexGuard<'_, SharedData>, DialogResult);

impl HomeScreen {
    fn show(&self) {
        self.widget.show_all();
    }
    fn hide(&self) {
        self.widget.hide();
    }
    fn process_keyboard_reply(
        shared_data: &std::sync::MutexGuard<SharedData>, // already locked!
        returnbutton: DialogResult,
    ) {
        let virtual_keyboard = &shared_data.virtual_keyboard.as_ref().unwrap();
        let binding = &shared_data.home_screen.lock().unwrap();
        let home_screen = binding.as_ref().unwrap();

        match returnbutton {
            DialogResult::Ok => {
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
                    virtual_keyboard.show(move |shared, returnbutton| {
                        println!("must now close kb dialog");
                        Self::process_keyboard_reply(shared, returnbutton);
                    });
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

struct VirtualKeyboard {
    widget: gtk::Box,
    input: Mutex<String>,
    accept: Mutex<String>,
    close_action: Mutex<DialogCloseAction>,
    display_label: Label,
    cursor_state: Mutex<bool>,
}

impl VirtualKeyboard {
    fn pre_cursor(&self, input: &str, cursorpos: usize) -> Option<String>
    {
        // given a string and a cursor position,
        // returns the portion of the string before the cursor
        let result = if input.len() > cursorpos {
            Some(input[..cursorpos].to_string())
        } else {
            if input.to_string() == ""
            {
                None
            } else {
                Some(input.to_string())
            }
        };
        result
    }
    fn on_cursor(&self, input: &str, cursorpos: usize) -> Option<String>
    {
        // given a string and a cursor position,
        // returns the portion of the string on the cursor
        let result = if cursorpos < input.len() {
            Some(input[cursorpos..cursorpos+1].to_string())
        } else {
            None
        };
        result
    }
    fn post_cursor(&self, input: &str, cursorpos: usize) -> Option<String>
    {
        // given a string and a cursor position,
        // returns the portion of the string after the cursor
        if input.len() > cursorpos+1 {
            Some(input[cursorpos+1..].to_string())
        } else {
            None
        }
    }
    fn update_label(&self, cursor: Option<&str>) {
        let mut cs = self.cursor_state.lock().expect("poison");
        let cursorshape = if let Some(c) = cursor {
            c
        } else {
            "_"
        };
        let input: &str = &self.input.lock().expect("poison");
        let mut cursorpos = input.len();  // but can be anything from 0..input.len() for edits

        // This IF shows that we can have a cursor underneath existing text
           if cursorpos >= 3 {
               cursorpos = 3;
           }
        let csh = if cursorshape == "_" {
            let insertmode: bool = false;
            // markup is not html but "Pango"
            let cursor_decoration_pre: &str = if insertmode {
                "<span foreground=\"white\" background=\"black\">"
            } else { "<u>" };
            let cursor_decoration_post: &str = if insertmode {
                "</span>"
            } else { "</u>" };
            format!("{}{}{}{}{}",
                self.pre_cursor(input, cursorpos).unwrap_or("".to_string()).replace("<", "&lt;"),
                cursor_decoration_pre,
                self.on_cursor(input, cursorpos).unwrap_or(" ".to_string()).replace("<", "&lt;"),
                cursor_decoration_post,
                self.post_cursor(input, cursorpos).unwrap_or("".to_string()).replace("<", "&lt;"),
            )
        } else {
            let filler = if self.on_cursor(input, cursorpos).is_none() {
                " "
            } else { "" }.to_string();
            format!("{}{}", input.to_string().replace("<", "&lt;"),
            filler
            )
        };
        self.display_label.set_markup(&csh);
    }
    fn blink_cursor(shared_data: &Arc<Mutex<SharedData>>) {
        let sd = shared_data.lock().expect("poison");
        let virtual_keyboard = sd.virtual_keyboard.as_ref();
        if let Some(vk) = virtual_keyboard {
            let mut cs = *vk.cursor_state.lock().expect("poison");
            {
                *vk.cursor_state.lock().expect("poison") = !cs;
            }
            vk.update_label(Some(if cs {"_"} else {" "}));
        }
    }

    fn append_input(&self, input: &str) {
        {
            let mut input_field = self.input.lock().expect("poison");
            let new_input = format!("{}{}", input_field, input);
            *input_field = new_input;
        }
        self.update_label(None);
    }

    fn reset_input(&self) {
        {
            let mut input_field = self.input.lock().expect("poison");
            let new_input = "".to_string();
            *input_field = new_input;
        }
        self.update_label(None);
    }

    fn show(&self, close_action: DialogCloseAction) {
        *self.close_action.lock().expect("poison") = close_action;
        self.widget.show_all();
    }

    fn hide(&self) {
        self.widget.hide();
    }
    fn button_callback(button: &gtk::Button, shared_data: &Arc<Mutex<SharedData>>) {
        // handles keyboard button mouse clicks, mostly.
        let button_label = button.label().unwrap();
        let shared = shared_data.lock().expect("poison");
        let virtual_keyboard = shared.virtual_keyboard.as_ref().unwrap();
        if button_label == "OK" {
            virtual_keyboard.hide();
            let action = virtual_keyboard.close_action.lock().expect("poison");
            action(&shared, DialogResult::Ok);
            return;
        }
        if button_label == "Cancel" {
            virtual_keyboard.hide();
            let action = virtual_keyboard.close_action.lock().expect("poison");
            action(&shared, DialogResult::Cancel);
            return;
        }
        // any other button on the dialog
        virtual_keyboard.append_input(&button_label);
    }
    fn _create_widget(shared_data: Arc<Mutex<SharedData>>, label: &Label) -> gtk::Box {
        // define the button event handler
        let shared_callback = move |button: &gtk::Button| {
            Self::button_callback(button, &shared_data);
        };

        // draw the keyboard

        let button_a = Button::with_label("A");
        button_a.connect_clicked(shared_callback.clone());
        //button_a.connect("key_press_event", false, |values| {println!("Button a!"); return true;} );
        let button_b = Button::with_label("OK");
        button_b.connect_clicked(shared_callback.clone());
        let button_c = Button::with_label("Cancel");
        button_c.connect_clicked(shared_callback.clone());

        let virtual_keyboard = gtk::Box::new(gtk::Orientation::Vertical, 5);
        virtual_keyboard.pack_start(label, true, true, 0);
        virtual_keyboard.pack_start(&button_a, true, true, 0);
        virtual_keyboard.pack_start(&button_b, true, true, 0);
        virtual_keyboard.pack_start(&button_c, true, true, 0);
        virtual_keyboard
    }
    fn new(shared_data: Arc<Mutex<SharedData>>) -> VirtualKeyboard {
        let display_label = gtk::Label::builder().name("display_label").build();
        // only a very limited set of tags is supported by this
        //display_label.set_markup("please type <b>SOMETHING</b>");

        let widget = VirtualKeyboard::_create_widget(Arc::clone(&shared_data), &display_label);
        let instance = VirtualKeyboard {
            widget,
            input: Mutex::new("".to_string()),
            accept: Mutex::new("".to_string()),
            close_action: Mutex::new(|_, _| {}),
            display_label,
            cursor_state: Mutex::new(false),
        };
        let shared_data_for_cursor = Arc::clone(&shared_data);
        // cursor blink timer thread
        let _source_id = glib::timeout_add_local(std::time::Duration::from_millis(400), move || {
            VirtualKeyboard::blink_cursor(&shared_data_for_cursor);
            Continue(true)
        });

        instance
    }
}

fn main() {
    gtk::init().expect("Failed to initialize GTK.");
    let vbox_main = gtk::Box::new(gtk::Orientation::Vertical, 5);

    let shared_data = Arc::new(Mutex::new(SharedData::new()));

    let virtual_keyboard = VirtualKeyboard::new(Arc::clone(&shared_data));
    let home_screen = HomeScreen::new(Arc::clone(&shared_data));
    vbox_main.pack_start(&home_screen.widget, true, true, 0);
    vbox_main.pack_start(&virtual_keyboard.widget, true, true, 0);

    shared_data.lock().expect("poison").virtual_keyboard = Some(virtual_keyboard);
    shared_data.lock().expect("poison").home_screen = Mutex::new(Some(home_screen));

    let shareddata_for_timer = Arc::clone(&shared_data);

    /*
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
        .load_from_data(
            "button { font-family: 'Arial'; font-size: 30px; font-weight: bold; } \
            #display_label { font-family: 'Courier'; font-size: 30px; font-weight: normal; } \
            ".as_bytes(),
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
