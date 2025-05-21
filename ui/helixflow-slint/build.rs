fn main() {
    let mut slint_config = slint_build::CompilerConfiguration::new();

    if std::env::var("DEBUG") == Ok("true".to_string()) {
        println!("cargo:rustc-env=SLINT_EMIT_DEBUG_INFO=1"); // for `slint!` macro

        use glob::glob;
        slint_config = slint_config.with_debug_info(true);
        for slint_file in glob("src/*.slint").expect("Failed to find slint files") {
            slint_build::compile_with_config(slint_file.unwrap(), slint_config.clone()).unwrap();
        }
    }

    slint_build::compile_with_config("src/tasks.slint", slint_config).unwrap();
}
