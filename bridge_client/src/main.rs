use std::io::{Read, Write};
use std::os::unix::net::UnixStream;
use std::env;

use bridge_core::{BridgeCommand, BridgeResponse};

// Lokasi socket dilihat dari sisi Chroot
// Asumsi: folder /data/local/tmp/chroot di-mount sebagai / (root) atau akses relatif
// Jika di dalam Chroot ada folder /tmp yang mapping ke host, maka:
const SOCKET_PATH: &str = "/tmp/bridge.sock";

fn main() -> std::io::Result<()> {
    // Ambil argumen dari CLI (contoh: ./client "screencap -p")
    // Untuk contoh input: andro input tap 500 500
    // args[0] = andro
    // args[1] = input (program)
    // args[2..] = tap, 500, 500 (arguments)

    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: andro <program> [args...]");
        eprintln!("Example: andro input tap 500 500");
        eprintln!("Example: andro pm list packages");
        return Ok(());
    }

    // Pisahkan nama program dan argumennya
    let program = args[1].clone();
    let program_args = args[2..].to_vec();

    // Bungkus dalam protokol Exec
    let command = BridgeCommand::Exec {
        program,
        args: program_args
    };

    // Kirim
    let mut stream = UnixStream::connect(SOCKET_PATH).expect("Server mati/socket tidak ditemukan");
    let json_payload = serde_json::to_vec(&command).unwrap();
    stream.write_all(&json_payload)?;

    // Baca Balikan (Response)
    let mut response_str = String::new();
    stream.read_to_string(&mut response_str)?;

    let response: BridgeResponse = serde_json::from_str(&response_str).unwrap();

    match response {
        BridgeResponse::Success(out) => print!("{}", out), // Print stdout
        BridgeResponse::Error(err) => eprintln!("Error: {}", err),
    }

    Ok(())
}
