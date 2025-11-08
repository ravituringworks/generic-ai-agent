#[cfg(feature = "tauri")]
fn main() {
    tauri_build::build();
}

#[cfg(not(feature = "tauri"))]
fn main() {
    // No Tauri build steps needed
    println!("No Tauri build steps needed");
}
