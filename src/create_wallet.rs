use gtk::prelude::*;
use gtk::{Button, Entry, Label, Orientation, Window};
use glib::clone;
use rusqlite::{Connection, params};
use std::path::PathBuf;
use bip39::{Mnemonic, Language};
use rand::RngCore;
use bcrypt::{hash, DEFAULT_COST};
use dirs_next;

pub fn open_create_wallet_window() {
    let wallet_window = Window::builder()
        .title("Litebag v.0.0.1")
        .default_width(300)
        .default_height(200)
        .build();

    let vbox = gtk::Box::new(Orientation::Vertical, 10);

    let wallet_entry = Entry::builder()
        .placeholder_text("Nombre de tu wallet")
        .build();

    let next_button = Button::builder()
        .label("Siguiente")
        .build();

    vbox.append(&wallet_entry);
    vbox.append(&next_button);
    wallet_window.set_child(Some(&vbox));
    wallet_window.show();

    next_button.connect_clicked(clone!(@strong wallet_entry => move |_| {
        let wallet_name = wallet_entry.text().to_string();
        if wallet_name.is_empty() {
            println!("Por favor elige un nombre para tu wallet");
            return;
        }
        let home_dir = dirs_next::home_dir().expect("No se pudo obtener el directorio home");
        let wallet_path = home_dir.join("litebag").join(format!("{}.wallet", wallet_name));
        std::fs::create_dir_all(wallet_path.parent().unwrap()).expect("No se pudo crear el directorio litebag");
        open_seed_generation_window(wallet_name, wallet_path);
        wallet_window.close();
    }));
}

fn open_seed_generation_window(wallet_name: String, wallet_path: PathBuf) {
    let seed_window = Window::builder()
        .title("Litebag v.0.0.1")
        .default_width(400)
        .default_height(300)
        .build();

    let vbox = gtk::Box::new(Orientation::Vertical, 10);

    let seed_label = Label::new(Some("Tu frase SEED:"));
    let seed_phrase = generate_seed();
    let seed_display = Entry::new();
    seed_display.set_text(&seed_phrase.0); 
    seed_display.set_editable(false); 

    let next_button = Button::builder()
        .label("Siguiente")
        .build();

    vbox.append(&seed_label);
    vbox.append(&seed_display);
    vbox.append(&next_button);
    seed_window.set_child(Some(&vbox));
    seed_window.show();

    next_button.connect_clicked(clone!(@strong seed_phrase => move |_| {
        open_seed_confirmation_window(wallet_name.clone(), wallet_path.clone(), seed_phrase.clone());
        seed_window.close();
    }));
}

fn open_seed_confirmation_window(wallet_name: String, wallet_path: PathBuf, seed_phrase: (String, Vec<u8>)) {
    let confirm_window = Window::builder()
        .title("Litebag v.0.0.1")
        .default_width(400)
        .default_height(300)
        .build();

    let vbox = gtk::Box::new(Orientation::Vertical, 10);

    let confirm_label = Label::new(Some("Por favor, confirma tu frase SEED:"));
    let confirm_entry = Entry::new();

    let next_button = Button::builder()
        .label("Siguiente")
        .build();

    vbox.append(&confirm_label);
    vbox.append(&confirm_entry);
    vbox.append(&next_button);
    confirm_window.set_child(Some(&vbox));
    confirm_window.show();

    next_button.connect_clicked(clone!(@strong seed_phrase, @strong confirm_entry => move |_| {
        let entered_seed = confirm_entry.text().to_string();
        if entered_seed == seed_phrase.0 {
            open_password_window(wallet_name.clone(), wallet_path.clone(), seed_phrase.clone());
            confirm_window.close();
        } else {
            println!("Las frases SEED no coinciden!");
        }
    }));
}

fn open_password_window(wallet_name: String, wallet_path: PathBuf, seed_phrase: (String, Vec<u8>)) {
    let password_window = Window::builder()
        .title("Litebag v.0.0.1")
        .default_width(400)
        .default_height(300)
        .build();

    let vbox = gtk::Box::new(Orientation::Vertical, 10);

    let password_label = Label::new(Some("Pon una contraseña (opcional):"));
    let password_entry = Entry::builder()
        .placeholder_text("Confirmar contraseña")
        .visibility(false)
        .build();

    let create_button = Button::builder()
        .label("Crear Wallet")
        .build();

    vbox.append(&password_label);
    vbox.append(&password_entry);
    vbox.append(&create_button);
    password_window.set_child(Some(&vbox));
    password_window.show();

    create_button.connect_clicked(clone!(@strong password_entry => move |_| {
        let password = password_entry.text().to_string();
        let hashed_password = hash_password(&password).expect("No se pudo encriptar la contraseña");
        save_wallet_to_sqlite(&wallet_name, &wallet_path, &seed_phrase.0, &seed_phrase.1, &hashed_password);
        println!("Tu cartera de LTC se creó!");
        password_window.close();
        open_create_wallet_window();
    }));
}

fn generate_seed() -> (String, Vec<u8>) {
    let mut entropy = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut entropy);
    let mnemonic = Mnemonic::from_entropy_in(Language::English, &entropy)
        .expect("Fallado al generar mnemonic");
    let seed = mnemonic.to_seed("");
    (mnemonic.to_string(), seed.to_vec())
}

fn hash_password(password: &str) -> Result<String, bcrypt::BcryptError> {
    hash(password, DEFAULT_COST)
}

fn save_wallet_to_sqlite(name: &str, path: &PathBuf, details: &str, seed: &[u8], password: &str) {
    let conn = Connection::open(path).expect("Fallado al abrir database SQLite");

    conn.execute(
        "CREATE TABLE IF NOT EXISTS wallets (
            id INTEGER PRIMARY KEY,
            name TEXT NOT NULL,
            details TEXT NOT NULL,
            seed BLOB NOT NULL,
            password TEXT NOT NULL
        )",
        [],
    ).expect("Fallado al crear tabla");

    conn.execute(
        "INSERT INTO wallets (name, details, seed, password) VALUES (?1, ?2, ?3, ?4)",
        params![name, details, seed, password],
    ).expect("Fallado al insertar wallet");

    println!("Wallet guardada en {:?}", path);
}