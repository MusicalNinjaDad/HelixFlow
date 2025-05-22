use std::{env, fs, path::PathBuf};

fn main() {
    let mut slint_config = slint_build::CompilerConfiguration::new();

    if std::env::var("DEBUG") == Ok("true".to_string()) {
        println!("cargo:rustc-env=SLINT_EMIT_DEBUG_INFO=1"); // for `slint!` macro
        slint_config = slint_config.with_debug_info(true);
        build_all_slint_modules(&slint_config);
    }

    slint_build::compile_with_config("src/helixflow.slint", slint_config).unwrap();
}

fn build_all_slint_modules(slint_config: &slint_build::CompilerConfiguration) {
    use glob::glob;

    for slint_file in glob("src/*.slint").expect("Failed to find slint files") {
        let mut src_file = PathBuf::new();
        src_file.push(env::var("CARGO_MANIFEST_DIR").unwrap());
        src_file.push(slint_file.as_ref().unwrap());

        let mut rs_file = PathBuf::new();
        rs_file.push(env::var("OUT_DIR").unwrap());
        rs_file.push(slint_file.as_ref().unwrap().with_extension("rs"));

        fs::create_dir_all(rs_file.parent().unwrap()).unwrap();

        println!("Compiling: {:#?} to {:#?}", &src_file, &rs_file);
        slint_build::compile_with_output_path(src_file, rs_file, slint_config.clone()).unwrap();
    }
}
