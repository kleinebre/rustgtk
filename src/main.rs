use gtk;
use gtk::prelude::*;
use gtk::{Application, ApplicationWindow, Button, Image};
use gdk::gdk_pixbuf::Pixbuf;

fn main() {
    let application = Application::builder()
        .application_id("com.example.FirstGtkApp")
        .build();

    application.connect_activate(|app| {
        let window = ApplicationWindow::builder()
            .application(app)
            .title("First GTK Program")
            .default_width(350)
            .default_height(70)
            .build();

        let mut pixbuf = Pixbuf::from_file("resources/monalisa.jpg").unwrap();
        for x in 0..127 {
            pixbuf.put_pixel(x,x,0,255,0,255);
            pixbuf.put_pixel(127-x,x,255,0,255,255);
        }
        let image_profile = Image::from_pixbuf(Some(&pixbuf));

        let button = Button::builder().image(&image_profile).build();
	button.connect_clicked(|_| {
            eprintln!("Clicked!");
        });
        window.add(&button);

        window.show_all();
    });

    application.run();
}
