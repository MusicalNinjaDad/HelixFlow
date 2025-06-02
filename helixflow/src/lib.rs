use std::rc::Rc;

use helixflow_core::task::TaskList;
use helixflow_core::task::blocking::CRUD;
use log::debug;
use slint::ComponentHandle;

use helixflow_slint::HelixFlow;
use helixflow_slint::task::blocking::{create_task, create_task_in_backlog, load_backlog};
use helixflow_surreal::blocking::SurrealDb;

pub fn run_helixflow() {
    debug!("Starting HelixFlow...");
    let backend = Rc::new(SurrealDb::new().unwrap());
    // let backend = Rc::new(SurrealDb::connect("127.0.0.1:8010").unwrap());
    let helixflow = HelixFlow::new().unwrap();
    let hf = helixflow.as_weak();
    let be = Rc::downgrade(&backend);
    helixflow.on_create_task(create_task(hf, be));

    let backlog = TaskList::new("This week");
    backlog.create(backend.as_ref()).unwrap();
    helixflow.set_backlog(backlog.into());

    let hf = helixflow.as_weak();
    let be = Rc::downgrade(&backend);
    helixflow.on_load_backlog(load_backlog(hf, be));

    let hf = helixflow.as_weak();
    let be = Rc::downgrade(&backend);
    helixflow.on_create_backlog_task(create_task_in_backlog(hf, be));

    helixflow.invoke_load_backlog();
    helixflow.show().unwrap();
    slint::run_event_loop().unwrap();
    helixflow.hide().unwrap();
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen(start)]
pub fn wasm_start() {
    console_error_panic_hook::set_once();
    console_log::init_with_level(log::Level::Debug).expect("error initializing log");
    debug!("Running helixflow in wasm");
    // panic!("So long and thanks for all the fish")
    run_helixflow();
}
