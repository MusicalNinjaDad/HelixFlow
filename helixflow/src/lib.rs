use std::rc::Rc;

use slint::ComponentHandle;

use helixflow_slint::{HelixFlow, create_task};
use helixflow_surreal::SurrealDb;

pub fn run_helixflow() {
    dbg!("Starting HelixFlow...");
    let backend = Rc::new(SurrealDb::create().unwrap());
    let helixflow = HelixFlow::new().unwrap();
    let hf = helixflow.as_weak();
    let be = Rc::downgrade(&backend);
    helixflow.on_create_task(create_task(hf, be));
    helixflow.show().unwrap();
    slint::run_event_loop().unwrap();
    helixflow.hide().unwrap();
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen(start)]
pub fn wasm_start() {
    console_error_panic_hook::set_once();
    console_log::init_with_level(log::Level::Debug).expect("error initializing log");
    run_helixflow();
}
