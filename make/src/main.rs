use std::process::{Command, Child};
use std::sync::Mutex;
use std::{thread, time::Duration};

static SERVER: Mutex<Option<Child>> = Mutex::new(None);

fn copy_folder(src: &str, dst: &str) {
    let dest = std::path::Path::new(dst);
    if dest.exists() {
        std::fs::remove_dir_all(dest).expect("failed to delete dest");
    }
    copy_dir_all(src, dest).unwrap_or_else(|e| panic!("failed to copy {} -> {}: {}", src, dst, e));
    println!("Copied {} -> {}", src, dst);
}

fn copy_dir_all(src: impl AsRef<std::path::Path>, dst: impl AsRef<std::path::Path>) -> std::io::Result<()> {
    std::fs::create_dir_all(&dst)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        if ty.is_dir() {
            copy_dir_all(entry.path(), dst.as_ref().join(entry.file_name()))?;
        } else {
            std::fs::copy(entry.path(), dst.as_ref().join(entry.file_name()))?;
        }
    }
    Ok(())
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
    let profile = if cfg!(debug_assertions) { "debug" } else { "release" };
    let mut args = vec!["build", "--target", "wasm32-unknown-unknown"];
    if !cfg!(debug_assertions) { args.push("--release"); }
    Command::new("cargo")
        .args(&args)
        .status()
        .unwrap();

    println!("Built wasm..");

    let wasm_path = format!("target/wasm32-unknown-unknown/{}/tcore.wasm", profile);

    Command::new("wasm-bindgen")
        .args(["--target", "web", "--out-dir", "web/pkg", &wasm_path])
        .status()
        .unwrap();

    println!("Binded wasm...");
}

fn main() {
    build();

    copy_folder("tcore/shaders", "web/shaders");
    copy_folder("tcore/img", "web/img");

    start_server();

    println!("Server started....");

    // keep process alive so server stays running
    loop {
        thread::sleep(Duration::from_secs(3600));
    }
}