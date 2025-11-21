use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
pub enum BridgeCommand {
    // Perintah Generic untuk menjalankan program binary Android Host apapun
    // program: nama binary (contoh: "input", "am", "pm", "ls")
    // args: daftar argumen
    Exec { program: String, args: Vec<String> },

    // tetap bisa simpan perintah khusus untuk utility lain
    Ping,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum BridgeResponse {
    Success(String), // Berisi stdout
    Error(String),   // Berisi stderr
}
