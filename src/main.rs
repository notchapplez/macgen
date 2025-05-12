use std::process::{Command, Stdio};
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::error::Error;
use serde_json::json;
use std::{fs::File, io::{Read, Write}, path::Path, sync::Arc};

type Result<T> = std::result::Result<T, Box<dyn Error>>;

#[derive(Serialize, Deserialize)]
struct Entry {
    client: String,
    mac: String,
}

const CREDENTIALS_FILE: &str = "/home/notchapplez/.cache/macgen_data.json";

#[derive(Serialize, Deserialize, Default)]
struct CredentialDB {
    credentials: Vec<Entry>,
}

fn load_db() -> CredentialDB {
    if Path::new(CREDENTIALS_FILE).exists() {
        let mut file = File::open(CREDENTIALS_FILE).unwrap();
        let mut data = String::new();
        file.read_to_string(&mut data).unwrap();
        serde_json::from_str(&data).unwrap_or_default()
    } else {
        CredentialDB::default()
    }
}

fn save_db(db: &CredentialDB) {
    let data = serde_json::to_string_pretty(db).unwrap();
    let mut file = File::create(CREDENTIALS_FILE).unwrap();
    file.write_all(data.as_bytes()).unwrap();
}

fn is_mac_registered(mac: &str) -> bool {
    let db = load_db();
    db.credentials.iter().any(|entry| entry.mac == mac)
}

fn generate_valid_mac() -> String {
    let mut rng = rand::rng();
    let first_byte = rng.random_range(0..16) << 4 | 0x02;
    let mut mac_bytes = vec![first_byte];
    for _ in 0..5 {
        mac_bytes.push(rng.random::<u8>());
    }
    mac_bytes.iter()
        .map(|b| format!("{:02x}", b))
        .collect::<Vec<String>>()
        .join(":")
}

fn run_cmd(cmd: &str, args: &[&str]) -> Result<()> {
    let output = Command::new(cmd)
        .args(args)
        .output()
        .map_err(|e| Box::new(e))?;
    if !output.status.success() {
        return Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!(
                "Command failed with error: {}",
                String::from_utf8_lossy(&output.stderr)
            ),
        )));
    }
    Ok(())
}

fn main() -> Result<()> {
    register_new_mac().expect(" ");
    Ok(())
}

fn check_mac()  {
    println!("Enter MAC address to check (format xx:xx:xx:xx:xx:xx):");
    let mut mac = String::new();
    std::io::stdin().read_line(&mut mac).expect("Failed to read line");
    let mac = mac.trim();

    if is_mac_registered(mac) {
        let db = load_db();
        if let Some(entry) = db.credentials.iter().find(|e| e.mac == mac) {
            println!("MAC {} is registered to client: {}", mac, entry.client);
        }
    } else {
        println!("MAC {} is not registered", mac);
    }
}

fn register_new_mac() -> Result<()> {
    if let Err(e) = run_cmd("ip", &["a"]) {
        println!("Warning: Could not display network interfaces: {}", e);
    }

    println!("Enter the wifi name to connect to:");
    let mut connection = String::new();
    std::io::stdin().read_line(&mut connection).expect("failed to read line");
    connection = connection.trim().to_string();

    let mut name = String::new();
    println!("Give your new mac a name, e.g name of client.");
    std::io::stdin().read_line(&mut name).expect("failed to read line");
    name = name.trim().to_string();

    let mut random_mac;
    loop {
        random_mac = generate_valid_mac();
        if !is_mac_registered(&random_mac) {
            break;
        }
        println!("Generated MAC already exists, trying another one...");
    }

    match run_cmd("nmcli", &["connection", "down", &connection]) {
        Ok(_) => println!(" "),
        Err(e) => println!(" "),
    }

    std::thread::sleep(std::time::Duration::from_secs(1));

    match run_cmd("nmcli", &["connection", "modify", &connection, "802-11-wireless.cloned-mac-address", &random_mac]) {
        Ok(_) => println!(" "),
        Err(e) => println!(" "),
    }

    std::thread::sleep(std::time::Duration::from_secs(1));

    match run_cmd("nmcli", &["connection", "up", &connection]) {
        Ok(_) => println!(" "),
        Err(e) => println!(""),
    }

    match run_cmd("nmcli", &["connection", "modify", &connection, "802-11-wireless.cloned-mac-address", " "]) {
        Ok(_) => println!(" "),
        Err(e) => println!(" "),
    }

    let mut db = load_db();
    let entry = Entry {
        client: name.clone(),
        mac: random_mac.clone(),
    };
    db.credentials.push(entry);
    save_db(&db);

    let json = json!({
        "name": name,
        "mac": random_mac,
    });
    println!("Client registered MAC address: {}", json);
    Ok(())
}