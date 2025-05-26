fn main() {
    let mut slint_config = slint_build::CompilerConfiguration::new();

    if std::env::var("DEBUG") == Ok("true".to_string()) {
        println!("cargo:rustc-env=SLINT_EMIT_DEBUG_INFO=1"); // for `slint!` macro
        slint_config = slint_config.with_debug_info(true);
    }

    slint_build::compile_with_config("src/helixflow.slint", slint_config).unwrap();
}
