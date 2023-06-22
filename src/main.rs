extern crate gtk;
use glib;
use gtk::builders::LabelBuilder;
use gtk::prelude::*;
use gtk::{Button, CssProvider, Label, Window, WindowType};
use std::sync::Arc;
use std::sync::Mutex;

pub const SCREEN_WIDTH: i32 = 800;
pub const SCREEN_HEIGHT: i32 = 480;
pub const SYMBOL_ENTER: &str = "✔";
pub const SYMBOL_CANCEL: &str = "🗙";
pub const SYMBOL_BACKSPACE: &str = "⌫";
//"↵";

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
    screen: Label,
    cursor_state: Mutex<bool>,
}
// key width, special key name, labels
type KeyDef = (f32, String, (String, String, String));
impl VirtualKeyboard {
    fn pre_cursor(&self, input: &str, cursorpos: usize) -> Option<String> {
        // given a string and a cursor position,
        // returns the portion of the string before the cursor
        let result = if input.len() > cursorpos {
            Some(input[..cursorpos].to_string())
        } else {
            if input.to_string() == "" {
                None
            } else {
                Some(input.to_string())
            }
        };
        result
    }
    fn on_cursor(&self, input: &str, cursorpos: usize) -> Option<String> {
        // given a string and a cursor position,
        // returns the portion of the string on the cursor
        let result = if cursorpos < input.len() {
            Some(input[cursorpos..cursorpos + 1].to_string())
        } else {
            None
        };
        result
    }
    fn post_cursor(&self, input: &str, cursorpos: usize) -> Option<String> {
        // given a string and a cursor position,
        // returns the portion of the string after the cursor
        if input.len() > cursorpos + 1 {
            Some(input[cursorpos + 1..].to_string())
        } else {
            None
        }
    }
    fn update_label(&self, cursor: Option<&str>) {
        let mut cs = self.cursor_state.lock().expect("poison");
        let cursorshape = if let Some(c) = cursor { c } else { "_" };
        let input: &str = &self.input.lock().expect("poison");
        let mut cursorpos = input.len(); // but can be anything from 0..input.len() for edits

        /* This IF shows that we can have a cursor underneath existing text
           if cursorpos >= 3 {
               cursorpos = 3;
           }
        */
        let csh = if cursorshape == "_" {
            let insertmode: bool = false;
            // markup is not html but "Pango"
            let cursor_decoration_pre: &str = if insertmode {
                "<span foreground=\"white\" background=\"black\">"
            } else {
                "<u>"
            };
            let cursor_decoration_post: &str = if insertmode { "</span>" } else { "</u>" };
            format!(
                "{}{}{}{}{}",
                self.pre_cursor(input, cursorpos)
                    .unwrap_or("".to_string())
                    .replace("<", "&lt;"),
                cursor_decoration_pre,
                self.on_cursor(input, cursorpos)
                    .unwrap_or(" ".to_string())
                    .replace("<", "&lt;"),
                cursor_decoration_post,
                self.post_cursor(input, cursorpos)
                    .unwrap_or("".to_string())
                    .replace("<", "&lt;"),
            )
        } else {
            let filler = if self.on_cursor(input, cursorpos).is_none() {
                " "
            } else {
                ""
            }
            .to_string();
            format!("{}{}", input.to_string().replace("<", "&lt;"), filler)
        };
        self.screen.set_markup(&csh);
    }
    fn blink_cursor(shared_data: &Arc<Mutex<SharedData>>) {
        let sd = shared_data.lock().expect("poison");
        let virtual_keyboard = sd.virtual_keyboard.as_ref();
        if let Some(vk) = virtual_keyboard {
            let mut cs = *vk.cursor_state.lock().expect("poison");
            {
                *vk.cursor_state.lock().expect("poison") = !cs;
            }
            vk.update_label(Some(if cs { "_" } else { " " }));
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

    fn backspace(&self) {
        {
            let mut input_field = self.input.lock().expect("poison");
            if input_field.len() > 0 {
                *input_field = input_field[0..input_field.len() - 1].to_string();
            }
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
        //let button_name = button.name().unwrap();
        let shared = shared_data.lock().expect("poison");
        let virtual_keyboard = shared.virtual_keyboard.as_ref().unwrap();
        if button_label == SYMBOL_BACKSPACE {
            virtual_keyboard.backspace();
            return;
        }
        if button_label == SYMBOL_ENTER {
            virtual_keyboard.hide();
            let action = virtual_keyboard.close_action.lock().expect("poison");
            action(&shared, DialogResult::Ok);
            return;
        }
        if button_label == SYMBOL_CANCEL {
            virtual_keyboard.hide();
            let action = virtual_keyboard.close_action.lock().expect("poison");
            action(&shared, DialogResult::Cancel);
            return;
        }
        // any other button on the dialog
        virtual_keyboard.append_input(&button_label);
    }
    fn define_keysets() -> Vec<Vec<KeyDef>> {
        let mut keys: Vec<Vec<KeyDef>> = vec![];

        let mut row: Vec<KeyDef> = vec![
            (
                1.0,
                "".to_string(),
                ("q".to_string(), "Q".to_string(), "1".to_string()),
            ),
            (
                1.0,
                "".to_string(),
                ("w".to_string(), "W".to_string(), "2".to_string()),
            ),
            (
                1.0,
                "".to_string(),
                ("e".to_string(), "E".to_string(), "3".to_string()),
            ),
            (
                1.0,
                "".to_string(),
                ("r".to_string(), "R".to_string(), "4".to_string()),
            ),
            (
                1.0,
                "".to_string(),
                ("t".to_string(), "T".to_string(), "5".to_string()),
            ),
            (
                1.0,
                "".to_string(),
                ("y".to_string(), "Y".to_string(), "6".to_string()),
            ),
            (
                1.0,
                "".to_string(),
                ("u".to_string(), "U".to_string(), "7".to_string()),
            ),
            (
                1.0,
                "".to_string(),
                ("i".to_string(), "I".to_string(), "8".to_string()),
            ),
            (
                1.0,
                "".to_string(),
                ("o".to_string(), "O".to_string(), "9".to_string()),
            ),
            (
                1.0,
                "".to_string(),
                ("p".to_string(), "P".to_string(), "0".to_string()),
            ),
            (
                1.0,
                "".to_string(),
                ("-".to_string(), "_".to_string(), "¬".to_string()),
            ),
            (
                1.0,
                "".to_string(),
                ("+".to_string(), "=".to_string(), "€".to_string()),
            ),
            (
                1.5,
                "Backspace".to_string(),
                ("⌫".to_string(), "⌫".to_string(), "⌫".to_string()),
            ),
        ]
        .to_vec();
        keys.push(row.clone());
        row = [
            (
                1.0,
                "spacer".to_string(),
                ("".to_string(), "".to_string(), "".to_string()),
            ),
            (
                1.0,
                "".to_string(),
                ("a".to_string(), "A".to_string(), "!".to_string()),
            ),
            (
                1.0,
                "".to_string(),
                ("s".to_string(), "S".to_string(), "\"".to_string()),
            ),
            (
                1.0,
                "".to_string(),
                ("d".to_string(), "D".to_string(), "£".to_string()),
            ),
            (
                1.0,
                "".to_string(),
                ("f".to_string(), "F".to_string(), "$".to_string()),
            ),
            (
                1.0,
                "".to_string(),
                ("g".to_string(), "G".to_string(), "%".to_string()),
            ),
            (
                1.0,
                "".to_string(),
                ("h".to_string(), "H".to_string(), "^".to_string()),
            ),
            (
                1.0,
                "".to_string(),
                ("j".to_string(), "J".to_string(), "&".to_string()),
            ),
            (
                1.0,
                "".to_string(),
                ("k".to_string(), "K".to_string(), "*".to_string()),
            ),
            (
                1.0,
                "".to_string(),
                ("l".to_string(), "L".to_string(), "(".to_string()),
            ),
            (
                1.0,
                "".to_string(),
                (";".to_string(), ":".to_string(), ")".to_string()),
            ),
            (
                1.0,
                "".to_string(),
                ("@".to_string(), "'".to_string(), "#".to_string()),
            ),
        ]
        .to_vec();
        keys.push(row.clone());
        row = [
            (
                2.0,
                "Shift".to_string(),
                ("⇧".to_string(), "⇧".to_string(), "⇧".to_string()),
            ),
            (
                1.0,
                "".to_string(),
                ("z".to_string(), "Z".to_string(), "|".to_string()),
            ),
            (
                1.0,
                "".to_string(),
                ("x".to_string(), "X".to_string(), "{".to_string()),
            ),
            (
                1.0,
                "".to_string(),
                ("c".to_string(), "C".to_string(), "}".to_string()),
            ),
            (
                1.0,
                "".to_string(),
                ("v".to_string(), "V".to_string(), "[".to_string()),
            ),
            (
                1.0,
                "".to_string(),
                ("b".to_string(), "B".to_string(), "]".to_string()),
            ),
            (
                1.0,
                "".to_string(),
                ("n".to_string(), "N".to_string(), "<".to_string()),
            ),
            (
                1.0,
                "".to_string(),
                ("m".to_string(), "M".to_string(), ">".to_string()),
            ),
            (
                1.0,
                "".to_string(),
                (",".to_string(), "<".to_string(), "/".to_string()),
            ),
            (
                1.0,
                "".to_string(),
                (".".to_string(), ">".to_string(), "?".to_string()),
            ),
        ]
        .to_vec();
        keys.push(row.clone());
        row = [
            (
                3.0,
                "Cancel".to_string(),
                ("🗙".to_string(), "🗙".to_string(), "🗙".to_string()),
            ),
            (
                1.0,
                "spacer".to_string(),
                ("".to_string(), "".to_string(), "".to_string()),
            ),
            (
                1.0,
                "spacer".to_string(),
                ("".to_string(), "".to_string(), "".to_string()),
            ),
            (
                1.0,
                "Left".to_string(),
                ("◁".to_string(), "◁".to_string(), "◁".to_string()),
            ),
            (
                1.0,
                "spacer".to_string(),
                ("".to_string(), "".to_string(), "".to_string()),
            ),
            (
                8.0,
                "spacebar".to_string(),
                (" ".to_string(), " ".to_string(), " ".to_string()),
            ),
            (
                1.0,
                "spacer".to_string(),
                ("".to_string(), "".to_string(), "".to_string()),
            ),
            (
                1.0,
                "Right".to_string(),
                ("▷".to_string(), "▷".to_string(), "▷".to_string()),
            ),
            (
                1.0,
                "spacer".to_string(),
                ("".to_string(), "".to_string(), "".to_string()),
            ),
            (
                1.0,
                "".to_string(),
                ("\\".to_string(), "`".to_string(), "~".to_string()),
            ),
            (
                1.0,
                "spacer".to_string(),
                ("".to_string(), "".to_string(), "".to_string()),
            ),
            (
                1.0,
                "spacer".to_string(),
                ("".to_string(), "".to_string(), "".to_string()),
            ),
            (
                2.0,
                "Ok".to_string(),
                (
                    SYMBOL_ENTER.to_string(),
                    SYMBOL_ENTER.to_string(),
                    SYMBOL_ENTER.to_string(),
                ),
            ),
        ]
        .to_vec();
        keys.push(row.clone());
        keys
    }

    fn _create_widget(
        shared_data: Arc<Mutex<SharedData>>,
        prompt: &Label,
        screen: &Label,
    ) -> gtk::Box {
        // define the button event handler
        let shared_callback = move |button: &gtk::Button| {
            Self::button_callback(button, &shared_data);
        };

        // draw the keyboard
        let keys = Self::define_keysets();
        let mut keyrow: usize = 1;

        let mut rowframes: Vec<gtk::ActionBar> = vec![];
        let mut keyset: usize = 0;

        for row in keys {
            keyrow += 1;
            let mut rowframe = gtk::ActionBar::new();
            //gtk::Box::builder().build();
            //rowframe.set_orientation(gtk::Orientation::Horizontal);
            let mut keycol: usize = 0;
            for key in row {
                keycol += 1;
                let (width, name, labels) = key;
                let mut bgcolor = "#FFFFFF";
                let mut fgcolor = "#000000";
                let label = labels.0;
                /*if self.accept != "" {
                    if !(self.accept.contains(label)) {
                        if !(self.is_special_key(label)) {
                            bgcolor = "#DDDDDD";
                            fgcolor = "#999999";
                        }
                    }
                }*/
                let w: i32 = (width * 2.0) as i32;
                println!("w={}", w);
                if name == "spacer" {
                    let spacer_box = gtk::Image::new();
                    rowframe.pack_start(&spacer_box);
                } else {
                    let button = Button::builder()
                        .label(&label)
                        .name(name)
                        .width_request(w)
                        //.pack_direction(PackDirecion::Ltr)
                        .build();
                    button.connect_clicked(shared_callback.clone());
                    button.connect("key_press_event", false, |values| {
                        println!("Button a!");
                        return Some(true.into());
                    });
                    //rowframe.alignment(Align::Center);
                    rowframe.pack_start(&button);
                }
            }
            rowframes.push(rowframe);
        }
        /*
        let button_a = Button::with_label("A");
        button_a.connect_clicked(shared_callback.clone());
        //button_a.connect("key_press_event", false, |values| {println!("Button a!"); return true;} );
        let button_b = Button::with_label("OK");
        button_b.connect_clicked(shared_callback.clone());
        let button_c = Button::with_label("Cancel");
        button_c.connect_clicked(shared_callback.clone());
        */
        let virtual_keyboard = gtk::Box::new(gtk::Orientation::Vertical, 5);
        virtual_keyboard.pack_start(prompt, true, true, 0);
        virtual_keyboard.pack_start(screen, true, true, 0);

        for bar in rowframes {
            virtual_keyboard.pack_start(&bar, true, true, 0);
        }
        //virtual_keyboard.pack_start(&button_a, true, true, 0);
        //virtual_keyboard.pack_start(&button_b, true, true, 0);
        //virtual_keyboard.pack_start(&button_c, true, true, 0);
        virtual_keyboard
    }
    fn new(shared_data: Arc<Mutex<SharedData>>, prompt_text: &str) -> VirtualKeyboard {
        let prompt = gtk::Label::builder().name("prompt").build();
        let screen = gtk::Label::builder().name("screen").build();
        prompt.set_text(prompt_text);
        // only a very limited set of tags is supported by this
        //screen.set_markup("please type <b>SOMETHING</b>");

        let widget = VirtualKeyboard::_create_widget(Arc::clone(&shared_data), &prompt, &screen);
        let instance = VirtualKeyboard {
            widget,
            input: Mutex::new("".to_string()),
            accept: Mutex::new("".to_string()),
            close_action: Mutex::new(|_, _| {}),
            screen,
            cursor_state: Mutex::new(false),
        };
        let shared_data_for_cursor = Arc::clone(&shared_data);
        // cursor blink timer thread
        let _source_id =
            glib::timeout_add_local(std::time::Duration::from_millis(400), move || {
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

    let virtual_keyboard =
        VirtualKeyboard::new(Arc::clone(&shared_data), "Please enter some text.");
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
            #screen { font-family: 'Courier'; font-size: 30px; font-weight: normal; } \
            #prompt { font-family: 'Arial'; font-size: 30px; font-weight: bold; } \
            #spacebar { padding-left: 200px; }
            #spacer { margin-left: 50px; }
            "
            .as_bytes(),
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
