use gtk::prelude::*;
use gtk::{Button, Entry, FileChooserAction, FileChooserDialog, Orientation, ResponseType, Window, MessageDialog, MessageType, ButtonsType, Notebook, Label, TextView, TextBuffer};
use glib::clone;
use rusqlite::{Connection, params};
use std::path::PathBuf;
use std::rc::Rc;
use std::cell::RefCell;
use bcrypt::verify;
use bitcoin::secp256k1::Secp256k1;
use bitcoin::secp256k1::SecretKey;
use bitcoin::PrivateKey;
use bitcoin::Network;
use bitcoin::key::PublicKey;
use bitcoin::base58;
use bitcoin::hashes::{Hash, sha256d};
use std::time::{SystemTime, Duration, UNIX_EPOCH};
use rand::RngCore;
use tokio::runtime::Runtime;
use std::thread;
use crate::transaction_monitor::monitor_transactions;
use crate::database::get_latest_address;
use crate::models::Transaction; 


fn generate_litecoin_address() -> (SecretKey, String) {
    let secp = Secp256k1::new();
    let mut rng = rand::thread_rng();
    let mut secret_key_bytes = [0u8; 32];
    rng.fill_bytes(&mut secret_key_bytes);
    let secret_key = SecretKey::from_slice(&secret_key_bytes).expect("Error generando la clave privada");

    let private_key = PrivateKey::new(secret_key, Network::Bitcoin);

    let public_key = PublicKey::from_private_key(&secp, &private_key);

    let pubkey_hash = sha256d::Hash::hash(&public_key.to_bytes());
    let pubkey_hash_160 = &pubkey_hash[..20]; 

    let mut payload = vec![0x30];
    payload.extend_from_slice(pubkey_hash_160);

    let checksum = sha256d::Hash::hash(&payload);
    let checksum = &checksum[..4]; 

    let mut address_bytes = payload;
    address_bytes.extend_from_slice(checksum);

    let address = base58::encode(&address_bytes);

    (secret_key, address)
}

fn save_address_to_sqlite(path: &PathBuf, address: &str, secret_key: &SecretKey, expiration: SystemTime) -> Result<(), String> {
    let conn = Connection::open(path).map_err(|e| format!("FALLO AL ABRIR LA WALLET: {}", e))?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS addresses (
            id INTEGER PRIMARY KEY,
            address TEXT NOT NULL,
            secret_key BLOB NOT NULL,
            expiration TEXT NOT NULL
        )",
        [],
    ).map_err(|e| format!("FALLO AL CREAR TABLA: {}", e))?;

    let expiration_timestamp = expiration.duration_since(UNIX_EPOCH).unwrap().as_secs().to_string();

    conn.execute(
        "INSERT INTO addresses (address, secret_key, expiration) VALUES (?1, ?2, ?3)",
        params![address, secret_key.as_ref(), expiration_timestamp],
    ).map_err(|e| format!("FALLO AL INSERTAR address: {}", e))?;

    Ok(())
}

fn load_transactions_from_sqlite(path: &PathBuf) -> Result<Vec<Transaction>, String> {
    let conn = Connection::open(path).map_err(|e| format!("FALLO AL ABRIR WALLET: {}", e))?;

    let mut stmt = conn.prepare("SELECT hash, amount, confirmations, received FROM transactions ORDER BY received DESC")
    .map_err(|e| format!("FALLO AL ESCUCHAR TRANSACCIONES: {}", e))?;

    let transactions = stmt.query_map([], |row| {
        Ok(Transaction {
            hash: row.get(0)?,
            amount: row.get(1)?,
            confirmations: row.get(2)?,
            received: row.get(3)?,
            inputs: Vec::new(), 
            outputs: Vec::new(), 
        })
    }).map_err(|e| format!("FALLO AL LEER TRANSACCIONES: {}", e))?
      .collect::<Result<Vec<_>, _>>()
      .map_err(|e| format!("ERROR PROCESANDO TRANSACCIONES: {}", e))?;

    Ok(transactions)
}

fn show_error_dialog(parent: &Window, message: &str) {
    let dialog = MessageDialog::builder()
        .message_type(MessageType::Error)
        .buttons(ButtonsType::Ok)
        .text(message)
        .build();

    dialog.set_transient_for(Some(parent));
    dialog.connect_response(|dialog, _| dialog.close());
    dialog.show();
}

fn show_success_dialog(parent: &Window, message: &str) {
    let dialog = MessageDialog::builder()
        .message_type(MessageType::Info)
        .buttons(ButtonsType::Ok)
        .text(message)
        .build();

    dialog.set_transient_for(Some(parent));
    dialog.connect_response(|dialog, _| dialog.close());
    dialog.show();
}

pub fn start_transaction_monitoring(path: PathBuf) {
    thread::spawn(move || {
        let rt = Runtime::new().expect("FALLO AL CREAR TOKIO RUNTIME");
        rt.block_on(async {
            if let Ok(address) = get_latest_address(&path) {
                monitor_transactions(&address, &path).await;
            } else {
                eprintln!("FALLO AL INTENTAR MONITOREAR LA DIRECCIÓN.");
            }
        });
    });
}

pub fn open_access_wallet_window() {
    let access_window = Window::builder()
        .title("Litebag v0.0.1")
        .default_width(300)
        .default_height(200)
        .build();

    let vbox = gtk::Box::new(Orientation::Vertical, 10);

    let file_chooser_button = Button::builder()
        .label("Selecciona el archivo de tu wallet!")
        .build();

    let password_entry = Entry::builder()
        .placeholder_text("Contraseña (sólo si se aplica)")
        .visibility(false)
        .build();

    let access_button = Button::builder()
        .label("Acceder a tu billetera")
        .build();

    let wallet_path = Rc::new(RefCell::new(None::<PathBuf>));

    file_chooser_button.connect_clicked(clone!(@strong wallet_path => move |_| {
        let dialog = FileChooserDialog::builder()
            .title("Selecciona archivo wallet")
            .action(FileChooserAction::Open)
            .build();

        dialog.add_buttons(&[("Abrir", ResponseType::Accept), ("Cancelar", ResponseType::Cancel)]);
        dialog.connect_response(clone!(@strong wallet_path => move |dialog, response| {
            if response == ResponseType::Accept {
                if let Some(path) = dialog.file().and_then(|f| f.path()) {
                    *wallet_path.borrow_mut() = Some(path);
                    println!("Se seleccionó el archivo wallet: {:?}", wallet_path.borrow());
                }
            }
            dialog.close();
        }));
        dialog.show();
    }));

    access_button.connect_clicked(clone!(@strong wallet_path, @strong password_entry, @strong access_window => move |_| {
        let path = wallet_path.borrow().clone();
        if path.is_none() {
            show_error_dialog(&access_window, "Por favor selecciona un archivo wallet!");
            return;
        }

        let path = path.unwrap(); 
        let password = password_entry.text().to_string();
        if let Err(e) = load_wallet(&path, &password) {
            show_error_dialog(&access_window, &e);
        } else {
            show_success_dialog(&access_window, "Se entró a la billetera!");

            start_transaction_monitoring(path.clone());

            open_wallet_main_window(&path); 
            access_window.close();
        }
    }));

    vbox.set_valign(gtk::Align::Center);
    vbox.set_halign(gtk::Align::Center);

    vbox.append(&file_chooser_button);
    vbox.append(&password_entry);
    vbox.append(&access_button);
    access_window.set_child(Some(&vbox));
    access_window.show();
}

fn open_wallet_main_window(wallet_path: &PathBuf) {
    let main_window = Window::builder()
        .title("Litebag Wallet")
        .default_width(600)
        .default_height(400)
        .build();

    let notebook = Notebook::new();

    let request_box = gtk::Box::new(Orientation::Vertical, 10);
    request_box.set_valign(gtk::Align::Center);
    request_box.set_halign(gtk::Align::Center);

    let request_label = Label::new(Some("Recibir LTC"));
    let subtitle_label = Label::new(Some("Buenas. Soy n4vv4r en GitHub. Este proyecto es una DEMO. Esperad problemas. !! BETA !!"));

    let address_label = TextView::new();
    address_label.set_editable(false);
    address_label.set_cursor_visible(false);
    address_label.set_wrap_mode(gtk::WrapMode::WordChar);

    let generate_button = Button::builder()
        .label("Generar dirección")
        .build();

    request_box.append(&request_label);
    request_box.append(&subtitle_label);
    request_box.append(&address_label);
    request_box.append(&generate_button);
    notebook.append_page(&request_box, Some(&Label::new(Some("Solicitar"))));

    generate_button.connect_clicked(clone!(@strong address_label, @strong wallet_path => move |_| {
        let (secret_key, address) = generate_litecoin_address();
        let expiration = SystemTime::now() + Duration::from_secs(3600); // 1 hour expiration

        if let Err(e) = save_address_to_sqlite(&wallet_path, &address.to_string(), &secret_key, expiration) {
            println!("ERROR AL GUARDAR EN DIRECCION: {}", e);
        } else {
            let buffer = TextBuffer::new(None);
            buffer.set_text(&format!("Dirección actual: {}", address));
            address_label.set_buffer(Some(&buffer));
        }
    }));

    let history_box = gtk::Box::new(Orientation::Vertical, 10);
    history_box.set_valign(gtk::Align::Center);
    history_box.set_halign(gtk::Align::Center);

    let history_label = Label::new(Some("Historial de transacciones"));
    let transactions_list = gtk::ListBox::new();

    history_box.append(&history_label);
    history_box.append(&transactions_list);

    if let Ok(transactions) = load_transactions_from_sqlite(&wallet_path) {
        for tx in transactions {
            let amount_ltc = tx.amount as f64 / 100_000_000.0; 
            let label = Label::new(Some(&format!(
                "Hash: {}\nCantidad: {} LTC\nConfirmaciones: {}\nFecha: {}",
                tx.hash, amount_ltc, tx.confirmations, tx.received
            )));
            transactions_list.append(&label);
        }
    }

    notebook.append_page(&history_box, Some(&Label::new(Some("Historial"))));

    main_window.set_child(Some(&notebook));
    main_window.show();
}

fn load_wallet(path: &PathBuf, password: &str) -> Result<(), String> {
    let conn = Connection::open(path).map_err(|e| format!("FALLO AL ABRIR LA WALLET: {}", e))?;

    let mut stmt = conn.prepare("SELECT password FROM wallets LIMIT 1")
        .map_err(|e| format!("FALLO AL QUERY DE WALLET: {}", e))?;
    let stored_password: String = stmt.query_row([], |row| row.get(0))
        .map_err(|e| format!("FALLO AL LEER Password: {}", e))?;

    if !stored_password.is_empty() {
        if !verify(password, &stored_password).map_err(|e| format!("ERROR AL VERIFICAR PASSWORD: {}", e))? {
            return Err("Contraseña incorrecta.".to_string());
        }
    }

    println!("Se cargó una wallet de {:?}", path);
    Ok(())
}