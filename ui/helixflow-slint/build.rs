fn main() {
    // Set SLINT_EMIT_DEBUG_INFO=1 for tests, rust-analyzer, etc.
    if std::env::var("DEBUG") == Ok("true".to_string()) {
        println!("cargo:rustc-env=SLINT_EMIT_DEBUG_INFO=1");
    }
}
