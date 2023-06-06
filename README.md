Welcome to the RustGTK repo
---------------------------

This is mostly boilerplate code testing out how to put
together user interfaces with Rust and GTK.

There are examples in src/examples/ and I have gradually
expanded these.
Note that the examples in src/examples/ are snapshots
of `src/main.rs` so you'll need to copy them to src/main.rs
to make them work with `cargo run`.

* src/examples/load_and_draw_on_pixbuf.rs

Loads an image from file into a pixbuf.
Draws pixels to the pixbuf. 
Creates an image widget from the pixbuf
Puts that image on a clickable button

* src/examples/css_styling.css

Shows how to use CSS to style buttons.

* src/examples/local_timeout.rs

Demonstrates how to trigger periodic events tied into the GUI

* src/examples/shared_data.rs

Shows how to share data between callbacks, e.g. a button and a timer

