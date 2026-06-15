use std::process::{Command, Child};
use std::sync::Mutex;
use std::{thread, time::Duration};

static SERVER: Mutex<Option<Child>> = Mutex::new(None);


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
            "target/wasm32-unknown-unknown/release/tcore.wasm",
        ])
        .status()
        .unwrap();
}

fn main() {
    build();

    start_server();

    println!("=====dev environment ready=====");

    // keep process alive so server stays running
    loop {
        thread::sleep(Duration::from_secs(3600));
    }
}