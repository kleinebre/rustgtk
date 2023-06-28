use crate::modules::home_screen::SharedData;
extern crate gtk;
use glib;
use gtk::prelude::*;
use gtk::{Button, Label};
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
pub const ID_DISABLED: &str = "disabled";
pub const VIRTUAL_KEYBOARD_CSS: &str = ".keyboard_button { margin:0; padding:0; font-family: Verdana; border-radius:0; border: 1px solid #999999; font-size: 26px; font-weight: bold; } \
            .keyboard_button_disabled { color: #CCCCCC; margin:0; padding:0; font-family: Verdana; border-radius:0; border: 1px solid #999999; font-size: 26px; font-weight: bold; } \
            .keyboard_button_row { padding:0; margin: 0; border:0; background: #cccccc; } \
            .root { padding:0; margin: 0; border:0; background: #cccccc; } \
            #ok { color: #009900; } \
            #cancel { color: #ff0000; } \
            #delete { font-family: Verdana; font-size: 12px; font-weight: normal; color: #000000; } \
            #insert { font-family: Verdana; font-size: 12px; font-weight: normal; } \
            .insert_active { color: #ff0000; } \
            .insert_inactive { color: #000000; } \
            #screen { font-family: 'Monospace';background: #eeeeee; font-size: 30px; font-weight: bold; } \
            #prompt { font-family: 'Verdana'; font-size: 30px; font-weight: bold; background: #cccccc; color: #000000;} \
            ";
//"↵";

#[derive(Debug)]
pub enum DialogResult {
    Ok,
    Cancel,
}

type DialogCloseAction = fn(&std::sync::MutexGuard<'_, SharedData>, DialogResult);
/*struct SharedData {
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
*/
pub struct VirtualKeyboard {
    pub widget: gtk::Box,
    pub input: Mutex<String>,
    close_action: Mutex<DialogCloseAction>,
    prompt: Label,
    screen: Label,
    active_key_layer: Mutex<usize>,
    keys_layers: Vec<gtk::Box>,
    cursor_state: Mutex<bool>,
    insert_mode: Mutex<bool>,
    cursor_pos: Mutex<usize>,
    pub accept: String,
}
// key width, special key name, labels
type KeyDef = (f32, String, [String; 3]);
impl VirtualKeyboard {
    pub fn charlen(input: &str) -> usize {
        let mut result_len = 0;
        for (l, _c) in input.chars().enumerate() {
            result_len = l + 1;
        }
        return result_len;
    }

    pub fn pre_cursor(input: &str, cursor_pos: usize) -> Option<String> {
        // given a string and a cursor position,
        // returns the portion of the string before the cursor
        let result = if Self::charlen(input) > cursor_pos {
            let mut tempstring = "".to_string();
            for (l, c) in input.chars().enumerate() {
                if l == cursor_pos {
                    return Some(tempstring);
                }
                tempstring.push(c);
            }
            return Some("".to_string());
            //Some(input[..cursor_pos].to_string())
        } else {
            if input.to_string() == "" {
                None
            } else {
                Some(input.to_string())
            }
        };
        result
    }
    pub fn on_cursor(input: &str, cursor_pos: usize) -> Option<String> {
        // given a string and a cursor position,
        // returns the portion of the string on the cursor
        let mut temp_string = "".to_string();
        let result = if cursor_pos < Self::charlen(input) {
            temp_string.push(input.chars().nth(cursor_pos)?);
            return Some(temp_string);
        } else {
            None
        };
        result
    }
    pub fn post_cursor(input: &str, cursor_pos: usize) -> Option<String> {
        // given a string and a cursor position,
        // returns the portion of the string after the cursor
        let l = Self::charlen(input);
        if l > cursor_pos + 1 {
            let mut tempstring = "".to_string();
            for i in (cursor_pos + 1)..l {
                tempstring.push(input.chars().nth(i)?);
            }
            return Some(tempstring);
        } else {
            None
        }
    }
    fn update_label(&self, cursor: Option<&str>) {
        let cursorshape = if let Some(c) = cursor { c } else { "_" };
        let input: &str = &self.input.lock().expect("poison");
        //let mut cursor_pos = input.len(); // but can be anything from 0..input.len() for edits
        let cursor_pos: usize = *self.cursor_pos.lock().expect("poison");
        /* This IF shows that we can have a cursor underneath existing text
           if cursor_pos >= 3 {
               cursor_pos = 3;
           }
        */
        let csh = if cursorshape == "_" {
            let insertmode: bool = { *self.insert_mode.lock().expect("poison") };
            // markup is not html but "Pango"
            let cursor_decoration_pre: &str = if insertmode {
                "<span foreground=\"white\" background=\"black\">"
            } else {
                "<u>"
            };
            let cursor_decoration_post: &str = if insertmode { "</span>" } else { "</u>" };
            format!(
                "{}{}{}{}{}",
                Self::pre_cursor(input, cursor_pos)
                    .unwrap_or("".to_string())
                    .replace("<", "&lt;"),
                cursor_decoration_pre,
                Self::on_cursor(input, cursor_pos)
                    .unwrap_or(" ".to_string())
                    .replace("<", "&lt;"),
                cursor_decoration_post,
                Self::post_cursor(input, cursor_pos)
                    .unwrap_or("".to_string())
                    .replace("<", "&lt;"),
            )
        } else {
            let filler = if Self::on_cursor(input, cursor_pos).is_none() {
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
            let cs = *vk.cursor_state.lock().expect("poison");
            {
                *vk.cursor_state.lock().expect("poison") = !cs;
            }
            vk.update_label(Some(if cs { "_" } else { " " }));
        }
    }

    fn append_input(&self, input: &str) {
        {
            let cursor_pos: usize = *self.cursor_pos.lock().expect("poison");
            let mut input_field = self.input.lock().expect("poison");
            let insertmode: bool = { *self.insert_mode.lock().expect("poison") };
            let pre = Self::pre_cursor(&input_field, cursor_pos)
                .unwrap_or("".to_string())
                .replace("<", "&lt;");
            let onc = Self::on_cursor(&input_field, cursor_pos)
                .unwrap_or(" ".to_string())
                .replace("<", "&lt;");
            let post = Self::post_cursor(&input_field, cursor_pos)
                .unwrap_or("".to_string())
                .replace("<", "&lt;");

            let new_input = if insertmode {
                format!("{}{}{}{}", pre, input, onc, post)
            } else {
                format!("{}{}{}", pre, input, post)
            };
            *input_field = new_input;
        }
        {
            let mut cursorpos = self.cursor_pos.lock().expect("poison");
            *cursorpos += 1;
        }
        self.update_label(None);
    }

    fn del_input(&self) {
        {
            let cursor_pos: usize = *self.cursor_pos.lock().expect("poison");
            let mut input_field = self.input.lock().expect("poison");
            let pre = Self::pre_cursor(&input_field, cursor_pos)
                .unwrap_or("".to_string())
                .replace("<", "&lt;");
            let post = Self::post_cursor(&input_field, cursor_pos)
                .unwrap_or("".to_string())
                .replace("<", "&lt;");

            let new_input = { format!("{}{}", pre, post) };
            *input_field = new_input;
        }
        self.update_label(None);
    }

    fn move_cursor_left(&self) {
        {
            let mut cursorpos = self.cursor_pos.lock().expect("poison");
            if *cursorpos > 0 {
                *cursorpos -= 1;
            }
        }
        self.update_label(None); // to keep curor visible while moving it
    }

    fn move_cursor_right(&self) {
        {
            let input_field = self.input.lock().expect("poison");
            let mut cursorpos = self.cursor_pos.lock().expect("poison");
            if *cursorpos <= Self::charlen(&input_field) {
                *cursorpos += 1;
            }
        }
        self.update_label(None); // to keep curor visible while moving it
    }

    fn backspace(&self) {
        // This code is pretty ugly and inefficient (not that that matters for
        // single-line strings of reasonable finite length)
        // but it does the trick of doing a backspace correctly, even for unicode strings.
        {
            let mut input_field = self.input.lock().expect("poison");
            let cursor_pos: usize = *self.cursor_pos.lock().expect("poison");
            if cursor_pos == 0 {
                return;
            }
            let pre = Self::pre_cursor(&input_field, cursor_pos - 1)
                .unwrap_or("".to_string())
                .replace("<", "&lt;");
            let post = Self::post_cursor(&input_field, cursor_pos - 1)
                .unwrap_or("".to_string())
                .replace("<", "&lt;");

            let new_input = format!("{}{}", pre, post);
            *input_field = new_input;
        }
        self.move_cursor_left();
        self.update_label(None);
    }

    pub fn reset_input(&self) {
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
                layer.show_all();
            } else {
                layer.hide();
            }
            idx += 1;
        }
    }

    pub fn show(&self, close_action: DialogCloseAction) {
        *self.close_action.lock().expect("poison") = close_action;
        *self.active_key_layer.lock().expect("poison") = 0;
        self.widget.show();
        self.prompt.show();
        self.screen.show();
        self.show_active_key_layer();
    }

    pub fn hide(&self) {
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
        return "".to_string();
    }

    pub fn handle_key(
        &self,
        shared: &std::sync::MutexGuard<SharedData>,
        button_label: &str,
        special_button_name: &str,
    ) {
        if special_button_name == ID_DISABLED {
            return;
        }
        if special_button_name == ID_BACKSPACE {
            self.backspace();
            return;
        }

        if special_button_name == "" {
            self.append_input(button_label);
            return;
        }
        if special_button_name == ID_SHIFT {
            self.next_keyset();
            return;
        }
        if special_button_name == ID_LEFT {
            self.move_cursor_left();
            return;
        }
        if special_button_name == ID_RIGHT {
            self.move_cursor_right();
            return;
        }
        if special_button_name == ID_INSERT {
            let mut insmode = self.insert_mode.lock().expect("poison");
            *insmode = !*insmode;
            return;
        }
        if special_button_name == ID_DELETE {
            self.del_input();
            return;
        }
        if special_button_name == ID_ENTER {
            self.hide();
            let action = self.close_action.lock().expect("poison");
            action(shared, DialogResult::Ok);
            return;
        }
        if special_button_name == ID_CANCEL {
            self.hide();
            let action = self.close_action.lock().expect("poison");
            action(shared, DialogResult::Cancel);
            return;
        }
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
        virtual_keyboard.handle_key(&shared, &button_label, &special_button_name);
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
                [
                    SYMBOL_CANCEL.to_string(),
                    SYMBOL_CANCEL.to_string(),
                    SYMBOL_CANCEL.to_string(),
                ],
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
                "".to_string(),
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
        accept: &str,
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

        let mut rowframes: Vec<gtk::Box> = vec![];

        for keyset in 0..3 {
            let keys_layer = gtk::Box::new(gtk::Orientation::Vertical, 3);
            for row in &keys {
                let rowframe = gtk::Box::builder().name("keyrow").build();
                rowframe.set_width_request(SCREEN_WIDTH - (BORDER_WIDTH * 2));
                let style_context = rowframe.style_context();
                style_context.add_class("keyboard_button_row");
                for key in row {
                    let (width, name, labels) = key;
                    let label = labels[keyset].clone();

                    let w: i32 = (width * 32.0) as i32;
                    if name == "spacer" {
                        let spacer_box = gtk::Box::new(gtk::Orientation::Horizontal, 0);
                        spacer_box.set_width_request(w);
                        rowframe.pack_start(&spacer_box, false, false, 0);
                    } else {
                        let button = Button::builder()
                            .name(name.clone())
                            .width_request(w)
                            .build();
                        let mut disabled = false;
                        if accept != "" {
                            if !accept.contains(&label) {
                                if name == "" {
                                    disabled = true;
                                }
                            }
                        }
                        let button_label = Label::new(Some(&label));
                        button_label.set_width_request(w);
                        button.add(&button_label);

                        button.connect_clicked(shared_callback.clone());
                        let style_context = button.style_context();
                        if disabled {
                            style_context.add_class("keyboard_button_disabled");
                            button.set_property("name", ID_DISABLED.to_string());
                        } else {
                            style_context.add_class("keyboard_button");
                            button.set_property("name", name);
                        }

                        button.set_hexpand(true);
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
    pub fn new(
        shared_data: Arc<Mutex<SharedData>>,
        prompt_text: &str,
        accept: &str,
    ) -> VirtualKeyboard {
        let prompt = gtk::Label::builder().name("prompt").build();
        let screen = gtk::Label::builder().name("screen").build();
        // Note: If choosing a different font for the screen, be sure
        // it doesn't do ligatures, so that it won't merge letterings for e.g. ff, fi
        // into a single glyph.

        let mut keys_layers: Vec<gtk::Box> = vec![];

        prompt.set_text(prompt_text);
        // only a very limited set of tags is supported by this
        //screen.set_markup("please type <b>SOMETHING</b>");

        let widget = VirtualKeyboard::_create_widget(
            Arc::clone(&shared_data),
            &prompt,
            &accept,
            &screen,
            &mut keys_layers,
        );
        let instance = VirtualKeyboard {
            widget,
            input: Mutex::new("".to_string()),
            close_action: Mutex::new(|_, _| {}),
            screen,
            prompt,
            active_key_layer: 0.into(),
            keys_layers,
            accept: accept.to_string(),
            cursor_state: Mutex::new(false),
            insert_mode: Mutex::new(false),
            cursor_pos: Mutex::new(0),
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

#[test]
fn test_charlen() {
    assert_eq!(VirtualKeyboard::charlen("abc"), 3);
    assert_eq!(VirtualKeyboard::charlen("ab€"), 3);
}

#[test]
fn test_pre_cursor() {
    assert_eq!(
        VirtualKeyboard::pre_cursor("abcde", 0).unwrap_or("".to_string()),
        ""
    );
    assert_eq!(
        VirtualKeyboard::pre_cursor("abcde", 1).unwrap_or("".to_string()),
        "a"
    );
    assert_eq!(
        VirtualKeyboard::pre_cursor("€bcde", 1).unwrap_or("".to_string()),
        "€"
    );
    assert_eq!(
        VirtualKeyboard::pre_cursor("€bcde", 2).unwrap_or("".to_string()),
        "€b"
    );
    assert_eq!(
        VirtualKeyboard::pre_cursor("€bcde", 5).unwrap_or("".to_string()),
        "€bcde"
    );
    assert_eq!(
        VirtualKeyboard::pre_cursor("€bcde", 6).unwrap_or("".to_string()),
        "€bcde"
    );
}

#[test]
fn test_on_cursor() {
    assert_eq!(VirtualKeyboard::on_cursor("a€c€e", 0).unwrap(), "a");
    assert_eq!(VirtualKeyboard::on_cursor("a€c€e", 1).unwrap(), "€");
    assert_eq!(VirtualKeyboard::on_cursor("a€c€e", 2).unwrap(), "c");
    assert_eq!(VirtualKeyboard::on_cursor("a€c€e", 3).unwrap(), "€");
    assert_eq!(VirtualKeyboard::on_cursor("a€c€e", 4).unwrap(), "e");
    assert!(VirtualKeyboard::on_cursor("a€c€e", 5).is_none());
}

#[test]
fn test_post_cursor() {
    assert_eq!(VirtualKeyboard::post_cursor("a€c€e", 0).unwrap(), "€c€e");
    assert_eq!(VirtualKeyboard::post_cursor("a€c€e", 1).unwrap(), "c€e");
    assert_eq!(VirtualKeyboard::post_cursor("a€c€e", 2).unwrap(), "€e");
    assert_eq!(VirtualKeyboard::post_cursor("a€c€e", 3).unwrap(), "e");
    assert!(VirtualKeyboard::post_cursor("a€c€e", 4).is_none());
    assert!(VirtualKeyboard::post_cursor("a€c€e", 5).is_none());
}
