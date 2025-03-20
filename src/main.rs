use gtk::prelude::*;
use gtk::{Application, ApplicationWindow, Button, Orientation};
use glib::clone;

mod create_wallet;
mod access_wallet;
mod transaction_monitor;
mod models;
mod database;
fn main() {
    let app = Application::builder()
        .application_id("litebag.n4vv4r.LitecoinWallet")
        .build();

    app.connect_activate(|app| {
        let window = ApplicationWindow::builder()
            .application(app)
            .title("Litebag v.0.0.1")
            .default_width(600)
            .default_height(400)
            .build();

        let create_wallet_button = Button::builder()
            .label("Crear tu wallet")
            .build();

        let access_wallet_button = Button::builder()
            .label("Accede a tu wallet")
            .build();

        create_wallet_button.connect_clicked(clone!(@weak window => move |_| {
            create_wallet::open_create_wallet_window();
            window.hide(); 
        }));

        access_wallet_button.connect_clicked(clone!(@weak window => move |_| {
            access_wallet::open_access_wallet_window();
            window.hide(); 
        }));

        let container = gtk::Box::new(Orientation::Vertical, 10);
        container.append(&create_wallet_button);
        container.append(&access_wallet_button);
        window.set_child(Some(&container));

        window.show();
    });

    app.run();
}