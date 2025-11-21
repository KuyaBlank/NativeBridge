use std::io::{Read, Write};
use std::os::unix::net::UnixListener;
use std::process::Command;
use std::fs;
use std::path::Path;

use bridge_core::{BridgeCommand, BridgeResponse};

// Lokasi socket dillihat dari sisi Android Host
// Pastikan path ini mengarah ke folder yang bisa dibaca oleh Chroot
const SOCKET_PATH: &str = "/data/local/tmp/chrootubuntu/tmp/bridge.sock";

fn main() -> std::io::Result<()> {
    // Bersihkan socket lama jika ada
    if Path::new(SOCKET_PATH).exists() {
        fs::remove_file(SOCKET_PATH)?;
    }

    // Bind Socket
    let listener = UnixListener::bind(SOCKET_PATH)?;
    println!("Server Bridge aktif di: {}", SOCKET_PATH);

    // Ubah permission socket agar bisa dibaca/ditulis oleh user chroot
    // Untuk saat ini Rust std lib ribet untuk ubah permission, pakai Command chroot yang simple

Command::new("chmod").arg("777").arg(SOCKET_PATH).output()?;

    // Loop menerima koneksi
    for stream in listener.incoming() {
        match stream {
            Ok(mut socket) => {
                // Handle setiap koneksi
                handle_client(&mut socket);
            }
            Err(err) => {
                eprintln!("Gagal menerima koneksi: {}", err);
            }
        }
    }
    Ok(())
}

fn handle_client(socket: &mut std::os::unix::net::UnixStream) {
    let mut buffer = [0; 8192];

    // Baca pesan dari Client
    match socket.read(&mut buffer) {
        Ok(size) => {
            if size == 0 { return; } // Kosong

            let raw_data = &buffer[0..size];
            let request: Result<BridgeCommand, _> = serde_json::from_slice(raw_data);

            let response = match request {
                Ok(cmd) => execute_request(cmd),
                Err(e) => BridgeResponse::Error(format!("Invalid JSON: {}", e)),
            };

            let resp_bytes = serde_json::to_vec(&response).unwrap();
            let _ = socket.write_all(&resp_bytes);
        }
        Err(_) => {}
    }
}

fn execute_request(cmd: BridgeCommand) -> BridgeResponse {
    match cmd {
        // Logika Universal: untuk semua program
        BridgeCommand::Exec { program, args } => {
            println!("Exec: {} {:?}", program, args); // Logging di server

            let output = Command::new(&program)
                .args(args)
                .output();

            match output {
                Ok(o) => {
                    if o.status.success() {
                        BridgeResponse::Success(String::from_utf8_lossy(&o.stdout).to_string())
                    } else {
                        // Jika command gagal (exit code !=0), Kirim stderr
                        BridgeResponse::Error(String::from_utf8_lossy(&o.stderr).to_string())
                    }
                },
                Err(e) => BridgeResponse::Error(format!("Gagal menjalankan {}: {}", program, e)),
            }
        },
        BridgeCommand::Ping => BridgeResponse::Success("Pong!".to_string()),
    }
}
