use std::process::{Command, Child};
use std::sync::Mutex;
use std::{thread, time::Duration};

static SERVER: Mutex<Option<Child>> = Mutex::new(None);

fn stop_server() {
    let mut server = SERVER.lock().unwrap();

    if let Some(mut child) = server.take() {
        let _ = child.kill();
        let _ = child.wait();
        println!("🛑 stopped old server");
    }
}
fn start_server() {
    let child = Command::new("python3")
        .args(["-m", "http.server", "9999", "--directory", "web"])
        .spawn()
        .expect("failed to start server");

    *SERVER.lock().unwrap() = Some(child);

    println!("🚀 server running on http://localhost:9999");
}

fn build() {
    Command::new("cargo")
        .args(["build", "--target", "wasm32-unknown-unknown", "--release"])
        .status()
        .unwrap();

    Command::new("wasm-bindgen")
        .args([
            "--target",
            "web",
            "--out-dir",
            "web/pkg",
            "target/wasm32-unknown-unknown/release/my_wasm.wasm",
        ])
        .status()
        .unwrap();
}

fn main() {
    build();

    stop_server();
    start_server();

    println!("✅ dev environment ready");

    // keep process alive so server stays running
    loop {
        thread::sleep(Duration::from_secs(3600));
    }
}