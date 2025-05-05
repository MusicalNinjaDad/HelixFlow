//! This is where the executable runtime invokation lives. It will make a lot of use of the
//! HelixFlow library ;-)

use std::rc::Rc;

use slint::ComponentHandle;

use helixflow_surreal::SurrealDb;
use helixflow_slint::{create_task, HelixFlow};

fn main() {
    let backend = Rc::new(SurrealDb::create().unwrap());
    let helixflow = HelixFlow::new().unwrap();
    let hf = helixflow.as_weak();
    let be = Rc::downgrade(&backend);
    helixflow.on_create_task(create_task(hf, be));
    helixflow.show().unwrap();
    slint::run_event_loop().unwrap();
    helixflow.hide().unwrap();
}
