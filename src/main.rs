extern crate gtk;
use glib;
use gtk::builders::LabelBuilder;
use gtk::prelude::*;
use gtk::{Button, CssProvider, Label, Window, WindowType};
use std::sync::Arc;
use std::sync::Mutex;

pub const SCREEN_WIDTH: i32 = 800;
pub const BORDER_WIDTH: i32 = 4;
pub const SCREEN_HEIGHT: i32 = 480;
pub const SYMBOL_ENTER: &str = "✔";
pub const ID_ENTER: &str = "ok";
pub const SYMBOL_CANCEL: &str = "🗙";
pub const ID_CANCEL: &str = "cancel";
pub const SYMBOL_BACKSPACE: &str = "⌫";
pub const ID_BACKSPACE: &str = "backspace";
pub const SYMBOL_LEFT: &str = "◁";
pub const ID_LEFT: &str = "left";
pub const SYMBOL_RIGHT: &str = "▷";
pub const ID_RIGHT: &str = "right";
pub const SYMBOL_INSERT: &str = "Ins";
pub const ID_INSERT: &str = "insert";
pub const SYMBOL_DELETE: &str = "Del";
pub const ID_DELETE: &str = "delete";
pub const SYMBOL_SHIFT: &str = "⇧";
pub const ID_SHIFT: &str = "shift";
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
    prompt: Label,
    screen: Label,
    active_key_layer: Mutex<usize>,
    keys_layers: Vec<gtk::Box>,
    cursor_state: Mutex<bool>,
}
// key width, special key name, labels
type KeyDef = (f32, String, [String; 3]);
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
        // This code is pretty ugly and inefficient (not that that matters for
        // strings of reasonable finite length)
        // but it does the trick of doing a backspace correctly, even for unicode strings.
        {
            let mut input_field = self.input.lock().expect("poison");
            let mut stringlen: usize = 0;
            for (l, c) in input_field.chars().enumerate() {
                stringlen = l;
            }
            let mut x = "".to_string();
            for (charnum, c) in input_field.chars().enumerate() {
                if charnum == stringlen {
                    break;
                }
                x.push(c);
            }
            *input_field = x;
            //if input_field.len() > 0 {
            //*input_field.pop(); // = input_field[0..input_field.len() - 1].to_string();
            //}
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

    fn show_active_key_layer(&self) {
        let mut idx: usize = 0;
        for layer in &self.keys_layers {
            if idx == *self.active_key_layer.lock().expect("poison") {
                self.keys_layers[idx].show_all();
            } else {
                self.keys_layers[idx].hide();
            }
            idx += 1;
        }
    }

    fn show(&self, close_action: DialogCloseAction) {
        *self.close_action.lock().expect("poison") = close_action;
        *self.active_key_layer.lock().expect("poison") = 0;
        self.widget.show();
        self.prompt.show();
        self.screen.show();
        self.show_active_key_layer();
    }

    fn hide(&self) {
        self.widget.hide();
    }
    fn next_keyset(&self) {
        let active_layer: usize = *self.active_key_layer.lock().expect("poison");
        let new_layer = (active_layer + 1) % (self.keys_layers.len());
        *self.active_key_layer.lock().expect("poison") = new_layer;
        self.show_active_key_layer();
    }

    fn button_label_text(button: &gtk::Button) -> String {
        let child = button.child();
        if let Some(widget) = child {
            // {
            let labelwidget = widget.downcast_ref::<Label>();
            if let Some(label) = labelwidget {
                return label.text().to_string();
            }
        }
        print!("No label found");
        return "".to_string();
    }

    fn button_callback(button: &gtk::Button, shared_data: &Arc<Mutex<SharedData>>) {
        // handles keyboard button mouse clicks, mostly.
        // Our button contains a label which contains the text (so that button width
        // is kept fixed) so we need some trickery to read the button label.
        let button_label = Self::button_label_text(&button);
        //let button_name = button.name().unwrap();
        let shared = shared_data.lock().expect("poison");
        let virtual_keyboard = shared.virtual_keyboard.as_ref().unwrap();

        let name_property: glib::Value = button.property::<glib::Value>("name");
        let special_button_name = if let Ok(string_value) = name_property.get::<String>() {
            if string_value == "" {
                "".to_string()
            } else {
                // special button clicked!
                string_value
            }
        } else {
            "".to_string()
        };

        if special_button_name == "" {
            virtual_keyboard.append_input(&button_label);
            return;
        }

        if special_button_name == ID_BACKSPACE {
            virtual_keyboard.backspace();
            return;
        }
        if special_button_name == ID_SHIFT {
            virtual_keyboard.next_keyset();
            return;
        }
        if special_button_name == ID_LEFT {
            return;
        }
        if special_button_name == ID_RIGHT {
            return;
        }
        if special_button_name == ID_INSERT {
            return;
        }
        if special_button_name == ID_DELETE {
            return;
        }
        if special_button_name == ID_ENTER {
            virtual_keyboard.hide();
            let action = virtual_keyboard.close_action.lock().expect("poison");
            action(&shared, DialogResult::Ok);
            return;
        }
        if special_button_name == ID_CANCEL {
            virtual_keyboard.hide();
            let action = virtual_keyboard.close_action.lock().expect("poison");
            action(&shared, DialogResult::Cancel);
            return;
        }
    }

    fn define_keysets() -> Vec<Vec<KeyDef>> {
        let mut keys: Vec<Vec<KeyDef>> = vec![];

        let mut row: Vec<KeyDef> = vec![
            (
                0.5,
                "spacer".to_string(),
                ["".to_string(), "".to_string(), "".to_string()],
            ),
            (
                1.0,
                "".to_string(),
                ["q".to_string(), "Q".to_string(), "1".to_string()],
            ),
            (
                1.0,
                "".to_string(),
                ["w".to_string(), "W".to_string(), "2".to_string()],
            ),
            (
                1.0,
                "".to_string(),
                ["e".to_string(), "E".to_string(), "3".to_string()],
            ),
            (
                1.0,
                "".to_string(),
                ["r".to_string(), "R".to_string(), "4".to_string()],
            ),
            (
                1.0,
                "".to_string(),
                ["t".to_string(), "T".to_string(), "5".to_string()],
            ),
            (
                1.0,
                "".to_string(),
                ["y".to_string(), "Y".to_string(), "6".to_string()],
            ),
            (
                1.0,
                "".to_string(),
                ["u".to_string(), "U".to_string(), "7".to_string()],
            ),
            (
                1.0,
                "".to_string(),
                ["i".to_string(), "I".to_string(), "8".to_string()],
            ),
            (
                1.0,
                "".to_string(),
                ["o".to_string(), "O".to_string(), "9".to_string()],
            ),
            (
                1.0,
                "".to_string(),
                ["p".to_string(), "P".to_string(), "0".to_string()],
            ),
            (
                1.0,
                "".to_string(),
                ["-".to_string(), "_".to_string(), "¬".to_string()],
            ),
            (
                1.0,
                "".to_string(),
                ["+".to_string(), "=".to_string(), "€".to_string()],
            ),
            (
                2.0,
                ID_BACKSPACE.to_string(),
                ["⌫".to_string(), "⌫".to_string(), "⌫".to_string()],
            ),
        ]
        .to_vec();
        keys.push(row.clone());
        row = [
            (
                1.0,
                ID_DELETE.to_string(),
                [
                    SYMBOL_DELETE.to_string(),
                    SYMBOL_DELETE.to_string(),
                    SYMBOL_DELETE.to_string(),
                ],
            ),
            (
                1.0,
                "".to_string(),
                ["a".to_string(), "A".to_string(), "!".to_string()],
            ),
            (
                1.0,
                "".to_string(),
                ["s".to_string(), "S".to_string(), "\"".to_string()],
            ),
            (
                1.0,
                "".to_string(),
                ["d".to_string(), "D".to_string(), "£".to_string()],
            ),
            (
                1.0,
                "".to_string(),
                ["f".to_string(), "F".to_string(), "$".to_string()],
            ),
            (
                1.0,
                "".to_string(),
                ["g".to_string(), "G".to_string(), "%".to_string()],
            ),
            (
                1.0,
                "".to_string(),
                ["h".to_string(), "H".to_string(), "^".to_string()],
            ),
            (
                1.0,
                "".to_string(),
                ["j".to_string(), "J".to_string(), "&".to_string()],
            ),
            (
                1.0,
                "".to_string(),
                ["k".to_string(), "K".to_string(), "*".to_string()],
            ),
            (
                1.0,
                "".to_string(),
                ["l".to_string(), "L".to_string(), "(".to_string()],
            ),
            (
                1.0,
                "".to_string(),
                [";".to_string(), ":".to_string(), ")".to_string()],
            ),
            (
                1.0,
                "".to_string(),
                ["'".to_string(), "@".to_string(), "`".to_string()],
            ),
            (
                1.0,
                "".to_string(),
                ["#".to_string(), "~".to_string(), "#".to_string()],
            ),
            (
                1.0,
                ID_INSERT.to_string(),
                [
                    SYMBOL_INSERT.to_string(),
                    SYMBOL_INSERT.to_string(),
                    SYMBOL_INSERT.to_string(),
                ],
            ),
        ]
        .to_vec();
        keys.push(row.clone());
        row = [
            (
                1.75,
                ID_SHIFT.to_string(),
                [
                    SYMBOL_SHIFT.to_string(),
                    SYMBOL_SHIFT.to_string(),
                    SYMBOL_SHIFT.to_string(),
                ],
            ),
            (
                1.0,
                "".to_string(),
                ["z".to_string(), "Z".to_string(), "{".to_string()],
            ),
            (
                1.0,
                "".to_string(),
                ["x".to_string(), "X".to_string(), "}".to_string()],
            ),
            (
                1.0,
                "".to_string(),
                ["c".to_string(), "C".to_string(), "[".to_string()],
            ),
            (
                1.0,
                "".to_string(),
                ["v".to_string(), "V".to_string(), "]".to_string()],
            ),
            (
                1.0,
                "".to_string(),
                ["b".to_string(), "B".to_string(), "<".to_string()],
            ),
            (
                1.0,
                "".to_string(),
                ["n".to_string(), "N".to_string(), ">".to_string()],
            ),
            (
                1.0,
                "".to_string(),
                ["m".to_string(), "M".to_string(), "|".to_string()],
            ),
            (
                1.0,
                "".to_string(),
                [",".to_string(), "<".to_string(), ",".to_string()],
            ),
            (
                1.0,
                "".to_string(),
                [".".to_string(), ">".to_string(), ".".to_string()],
            ),
            (
                1.0,
                "".to_string(),
                ["/".to_string(), "?".to_string(), "\\".to_string()],
            ),
            (
                3.0,
                "spacer".to_string(),
                ["".to_string(), "".to_string(), "".to_string()],
            ),
        ]
        .to_vec();
        keys.push(row.clone());
        row = [
            (
                3.0,
                ID_CANCEL.to_string(),
                [SYMBOL_CANCEL.to_string(), SYMBOL_CANCEL.to_string(), SYMBOL_CANCEL.to_string()],
            ),
            (
                0.25,
                "spacer".to_string(),
                ["".to_string(), "".to_string(), "".to_string()],
            ),
            (
                1.0,
                ID_LEFT.to_string(),
                [
                    SYMBOL_LEFT.to_string(),
                    SYMBOL_LEFT.to_string(),
                    SYMBOL_LEFT.to_string(),
                ],
            ),
            (
                8.0,
                "spacebar".to_string(),
                [" ".to_string(), " ".to_string(), " ".to_string()],
            ),
            (
                1.0,
                ID_RIGHT.to_string(),
                [
                    SYMBOL_RIGHT.to_string(),
                    SYMBOL_RIGHT.to_string(),
                    SYMBOL_RIGHT.to_string(),
                ],
            ),
            (
                0.25,
                "spacer".to_string(),
                ["".to_string(), "".to_string(), "".to_string()],
            ),
            (
                3.0,
                ID_ENTER.to_string(),
                [
                    SYMBOL_ENTER.to_string(),
                    SYMBOL_ENTER.to_string(),
                    SYMBOL_ENTER.to_string(),
                ],
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
        keys_layers: &mut Vec<gtk::Box>,
    ) -> gtk::Box {
        // define the button event handler
        let shared_callback = move |button: &gtk::Button| {
            Self::button_callback(button, &shared_data);
        };
        screen.set_xalign(0.0);
        // draw the keyboard
        let keys = Self::define_keysets();
        let mut keyrow: usize = 1;

        let mut rowframes: Vec<gtk::Box> = vec![];

        for keyset in 0..3 {
            let keys_layer = gtk::Box::new(gtk::Orientation::Vertical, 3);
            for row in &keys {
                keyrow += 1;
                let mut rowframe = gtk::Box::builder().name("keyrow").build();
                rowframe.set_width_request(SCREEN_WIDTH - (BORDER_WIDTH * 2));
                let style_context = rowframe.style_context();
                style_context.add_class("keyboard_button_row");
                let mut keycol: usize = 0;
                for key in row {
                    keycol += 1;
                    let (width, name, labels) = key;
                    let mut bgcolor = "#FFFFFF";
                    let mut fgcolor = "#000000";
                    let label = labels[keyset].clone();
                    /*if self.accept != "" {
                        if !(self.accept.contains(label)) {
                            if !(self.is_special_key(label)) {
                                bgcolor = "#DDDDDD";
                                fgcolor = "#999999";
                            }
                        }
                    }*/
                    let w: i32 = (width * 32.0) as i32;
                    if name == "spacer" {
                        let spacer_box = gtk::Box::new(gtk::Orientation::Horizontal, 0);
                        spacer_box.set_width_request(w);
                        rowframe.pack_start(&spacer_box, false, false, 0);
                    } else {
                        let button = Button::builder().name(name).width_request(w).build();
                        let button_label = Label::new(Some(&label));
                        button_label.set_width_request(w);
                        button.add(&button_label);

                        button.connect_clicked(shared_callback.clone());
                        let style_context = button.style_context();
                        style_context.add_class("keyboard_button");
                        button.set_hexpand(true);
                        button.set_property("name", name);
                        //button.halign(gtk::Align::Fill);
                        //button.set_vexpand(true);
                        button.connect("key_press_event", false, |values| {
                            println!("Button a!");
                            return Some(true.into());
                        });
                        rowframe.pack_start(&button, false, true, 0);
                    }
                }
                rowframes.push(rowframe);
            }
            for bar in &rowframes {
                keys_layer.pack_start(bar, true, true, 0);
            }
            keys_layer.set_height_request(SCREEN_HEIGHT * 3 / 4);
            keys_layer.hide();
            keys_layers.push(keys_layer);
        }
        let virtual_keyboard = gtk::Box::new(gtk::Orientation::Vertical, 5);
        prompt.set_height_request(SCREEN_HEIGHT * 5 / 40);
        screen.set_height_request(SCREEN_HEIGHT * 5 / 40);

        virtual_keyboard.pack_start(prompt, true, true, 0);
        virtual_keyboard.pack_start(screen, true, true, 0);
        for keys_layer in keys_layers {
            virtual_keyboard.pack_start(keys_layer, true, true, 0);
        }
        virtual_keyboard.set_border_width(BORDER_WIDTH as u32);
        virtual_keyboard
    }
    fn new(shared_data: Arc<Mutex<SharedData>>, prompt_text: &str) -> VirtualKeyboard {
        let prompt = gtk::Label::builder().name("prompt").build();
        let screen = gtk::Label::builder().name("screen").build();
        let mut keys_layers: Vec<gtk::Box> = vec![];

        prompt.set_text(prompt_text);
        // only a very limited set of tags is supported by this
        //screen.set_markup("please type <b>SOMETHING</b>");

        let widget = VirtualKeyboard::_create_widget(
            Arc::clone(&shared_data),
            &prompt,
            &screen,
            &mut keys_layers,
        );
        let instance = VirtualKeyboard {
            widget,
            input: Mutex::new("".to_string()),
            accept: Mutex::new("".to_string()),
            close_action: Mutex::new(|_, _| {}),
            screen,
            prompt,
            active_key_layer: 0.into(),
            keys_layers,
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
    let style_context = vbox_main.style_context();
    style_context.add_class("root");
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
            ".keyboard_button { margin:0; padding:0; font-family: Verdana; border-radius:0; border: 1px solid #999999; font-size: 26px; font-weight: bold; } \
            .keyboard_button_row { padding:0; margin: 0; border:0; background: #cccccc; } \
            .root { padding:0; margin: 0; border:0; background: #cccccc; } \
            #ok { color: #009900; } \
            #cancel { color: #ff0000; } \
            #delete { font-family: Verdana; font-size: 12px; font-weight: normal; color: #000000; } \
            #insert { font-family: Verdana; font-size: 12px; font-weight: normal; color: #000000; } \
            #screen { font-family: 'Courier'; background: #eeeeee; font-size: 30px; font-weight: bold; } \
            #prompt { font-family: 'Verdana'; font-size: 30px; font-weight: bold; background: #cccccc; color: #000000;} \
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
