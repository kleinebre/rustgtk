# Welcome to the RustGTK repo

This is mostly boilerplate code testing out how to put together user interfaces with Rust and GTK.

There are examples in src/examples/ and I have gradually expanded these. Note that the examples in src/examples/ are snapshots of `src/main.rs` so you'll need to copy them to src/main.rs to make them work with `cargo run`.

- src/examples/load_and_draw_on_pixbuf.rs

Loads an image from file into a pixbuf. Draws pixels to the pixbuf. Creates an image widget from the pixbuf Puts that image on a clickable button

- src/examples/css_styling.css

Shows how to use CSS to style buttons.

- src/examples/local_timeout.rs

Demonstrates how to trigger periodic events tied into the GUI

- src/examples/shared_data.rs

Shows how to share data between callbacks, e.g. a button and a timer

- src/examples/shared_callback_code.rs

Defines a single closure, used as callback in several widgets

- src/examples/initial_dialog.rs

Opens a custom dialog window from another window. This is a bit rough around the edges, but some points are

- Only the active window is visible. This is fine taking account we're targeting an app where every window is fullscreen on a single touch screen. With both windows running but only one visible, we can act modal while the rest of the application is still active. A future version will have just 1 window and each dialog will be a box inside that window (with only 1 box visible).
- The "x" close button works correctly for the dialog to close itself. In this case the behaviour is the same as button "A" which is deemed the close/cancel button for now.

- src/examples/single_window_multi_dialog_interface.rs

As the previous one, but has a single-window UI and keeps the dialogs all in the same window; a better match for a full-screen touchscreen app. This also makes the dialog code more modular.

- src/examples/one_button_keyboard.rs

This shows how one might implement the most basic on-screen keyboard dialog. There's no display, no backspace, just one button and OK/Cancel. The result is handled with a callback.

- src/examples/struct_based_keyboard.rs

As above but now with all the structures properly defined.

- src/examples/key_handler.rs

Demonstrates how to intercept keyboard events
