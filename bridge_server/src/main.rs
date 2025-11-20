use std::io::{Read, Write};
use std::os::unix::net::UnixListener;
use std::process::Command;
use std::fs;
use std::path::Path;

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
    let mut buffer = [0; 1024];

    // Baca pesan dari Client
    match socket.read(&mut buffer) {
        Ok(size) => {
            if size == 0 { return; } // Kosong

            // Convert bytes ke String
            let command_str = String::from_utf8_lossy(&buffer[0..size]);
            let command_trim = command_str.trim();
            println!("Menerima perintah: {}", command_trim);

            // Eksekusi Command di Android System
            // Pakai 'sh -c' agar bisa parsing argumen kompleks
            let output = Command::new("sh")
                .arg("-c")
                .arg(command_trim)
                .output();

            match output {
                Ok(o) => {
                    // Kirim balik stdout (hasil sukses)
                    let _ = socket.write_all(&o.stdout);
                    // kirim balik stderr (jika ada error)
                    let _ = socket.write_all(&o.stderr);
                }
                Err(e) => {
                    let error_msg = format!("Gagal eksekusi: {}", e);
                    let _ = socket.write_all(error_msg.as_bytes());
                }
            }
        }
        Err(_) => {}
    }
}
