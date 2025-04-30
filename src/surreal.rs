use std::sync::LazyLock;

use anyhow::{Context, Ok};
use surrealdb::{engine::remote::ws::{Client, Ws}, opt::auth::Root, Object, Surreal, sql::Thing};

use tokio::runtime::Runtime;

use crate::task::{StorageBackend, Task};

/// A blocking instance of a SurrealDb
struct SurrealDb
// {
//     /// The async surrealdb Client
//     inner: Client,

//     /// A `current_thread` `tokio::Runtime`
//     rt: Runtime,
// }

// impl SurrealDb {
//     fn connect() -> anyhow::Result<Self> {
//         let rt = tokio::runtime::Builder::new_current_thread()
//         .enable_all()
//         .build()?;
        
//         let inner = ;
        

//     }
// }
;

impl StorageBackend<Thing> for SurrealDb {
    fn create(&self, task: &mut Task<Thing>) -> anyhow::Result<()> {
        let rt = tokio::runtime::Builder::new_current_thread().enable_all().build().context("Creating Tokio Runtime")?;
        let db =  rt.block_on(Surreal::new::<Ws>("localhost:8000").into_future()).context("Initialising database connection")?;
        rt.block_on(db.signin(Root{
            username: "root",
            password: "root",
        }).into_future()).context("Signing in to database")?;
        rt.block_on(db.use_ns("namespace").use_db("database").into_future()).context("Selecting database namespace")?;
        let _: Option<Task<Thing>> = rt.block_on(db.create("Tasks").content(Task::<Thing>{
            name: "Hardcoded".into(),
            id: None,
            description: None}
        ).into_future()).context("Creating new record")?;
        let tasks: Vec<Task<Thing>> = rt.block_on(
            db.select("Tasks").into_future()
        ).context("Reading records")?;
        task.name = "Hardcoded".into();
        task.id = tasks[1].id.clone();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_task() {
        {
            let mut new_task = Task {
                name: "Test Task 1".into(),
                id: None,
                description: None,
            };
            let backend = SurrealDb;
            let created = new_task.create(&backend);
            if let Err(e) = created {dbg!("Error: {}", e);};
            assert_eq!(new_task.name, "Hardcoded");
            assert_eq!(new_task.description, None);
            assert!(new_task.id != None);
        }
    }
}
