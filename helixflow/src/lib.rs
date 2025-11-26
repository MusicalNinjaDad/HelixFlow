#![feature(coverage_attribute)]
#![feature(if_let_guard)]
#![coverage(off)]
use std::{path::PathBuf, rc::Rc};

use log::debug;
use slint::ComponentHandle;

use helixflow_core::{CRUD, HelixFlowError, state::State, task::TaskList};
use helixflow_slint::{
    HelixFlow,
    task::{create_task, create_task_in_backlog, load_backlog},
};
use helixflow_surreal::SurrealDb;
use uuid::uuid;

pub fn run_helixflow() {
    debug!("Starting HelixFlow...");

    let mut db_file = PathBuf::new();
    db_file.push("helixflow.kv");

    let backend = Rc::new(SurrealDb::new(Some(db_file)).unwrap());
    let helixflow = HelixFlow::new().unwrap();

    let state_id = uuid!("867bb83c-730a-4470-9fcd-14359cf5292b");
    let mut ui_state = match State::get(backend.as_ref(), &state_id) {
        Ok(state) => state,
        Err(e) => match e {
            HelixFlowError::NotFound { itemtype, id } if itemtype == "State" && id == state_id => {
                State::new(&state_id)
            }
            _ => panic!("{}", e),
        },
    };

    let backlog = match ui_state.visible_backlog_id() {
        Some(id) => TaskList::get(backend.as_ref(), id).unwrap(),
        None => {
            let backlog = TaskList::new("This week");
            backlog.create(backend.as_ref()).unwrap();
            ui_state.visible_backlog(&backlog);
            // TODO implement an Update in CRUD and create State earlier ...
            ui_state.create(backend.as_ref()).unwrap();
            backlog
        }
    };
    helixflow.set_backlog(backlog.into());

    let hf = helixflow.as_weak();
    let be = Rc::downgrade(&backend);
    helixflow.on_load_backlog(load_backlog(hf, be));
    helixflow.invoke_load_backlog();

    let hf = helixflow.as_weak();
    let be = Rc::downgrade(&backend);
    helixflow.on_create_backlog_task(create_task_in_backlog(hf, be));

    let hf = helixflow.as_weak();
    let be = Rc::downgrade(&backend);
    helixflow.on_create_task(create_task(hf, be));

    helixflow.show().unwrap();
    slint::run_event_loop().unwrap();
    helixflow.hide().unwrap();
}
