//! This is where the executable runtime invokation lives. It will make a lot of use of the
//! HelixFlow library ;-)

use std::sync::Arc;

use slint::ComponentHandle;
use tokio::task::block_in_place;

use helixflow::backends::surreal::SurrealDb;
use helixflow::ui::slint::{create_task, HelixFlow};

#[tokio::main]
async fn main() {
    let backend = Arc::new(SurrealDb::create().await.unwrap());
    let helixflow = HelixFlow::new().unwrap();
    let hf = helixflow.as_weak();
    helixflow.on_create_task(create_task(hf, backend));
    helixflow.show().unwrap();
    block_in_place(slint::run_event_loop).unwrap();
    helixflow.hide().unwrap();
}
