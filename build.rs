fn main() {
    if std::env::var("CARGO_FEATURE_TAURI").is_ok() {
        tauri_build::build();
    }
}
