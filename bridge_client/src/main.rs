use std::io::{Read, Write};
use std::os::unix::net::UnixStream;
use std::env;

// Lokasi socket dilihat dari sisi Chroot
// Asumsi: folder /data/local/tmp/chroot di-mount sebagai / (root) atau akses relatif
// Jika di dalam Chroot ada folder /tmp yang mapping ke host, maka:
const SOCKET_PATH: &str = "/tmp/bridge.sock";

fn main() -> std::io::Result<()> {
    // Ambil argumen dari CLI (contoh: ./client "screencap -p")
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Gunakan: {} <perintah_android", args[0]);
        return Ok(());
    }

    let command_to_send = &args[1];

    // Connect ke Server
    let mut stream = UnixStream::connect(SOCKET_PATH).map_err(|e| {
        eprintln!("Gagal connect ke socket di {}. Pastikan server menyala!", SOCKET_PATH);
        e
    })?;

    // Kirim perintah
    stream.write_all(command_to_send.as_bytes())?;

    // Baca Balikan (Response)
    let mut response = String::new();
    stream.read_to_string(&mut response)?;

    // Cetak output persis seperti aslinya
    print!("{}", response);

    Ok(())
}
