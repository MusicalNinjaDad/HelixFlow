use glob::glob;

fn main() {
    // Set SLINT_EMIT_DEBUG_INFO=1 for tests, rust-analyzer, etc.
    if std::env::var("DEBUG") == Ok("true".to_string()) {
        println!("cargo:rustc-env=SLINT_EMIT_DEBUG_INFO=1");
    }
    let config = slint_build::CompilerConfiguration::new().with_debug_info();
    for slint_file in glob("src/*.slint").expect("Failed to find slint files") {
        slint_build::compile_with_config(slint_file.unwrap(), config.clone()).unwrap();
    }
}
