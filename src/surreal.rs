use anyhow::{Context, Ok};
use surrealdb::{engine::remote::ws::{Client, Ws}, opt::auth::Root, Surreal, sql::Thing};

use tokio::runtime::Runtime;

use crate::task::{StorageBackend, Task};

/// A blocking instance of a SurrealDb
struct SurrealDb
{
    /// The async surrealdb Client
    db: Surreal<Client>,

    /// A `current_thread` `tokio::Runtime`
    rt: Runtime,
}

impl SurrealDb {
    fn connect() -> anyhow::Result<Self> {
        let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .context("Creating Tokio Runtime")?;
        
        let db = rt.block_on(Surreal::new::<Ws>("localhost:8000").into_future()).context("Initialising database connection")?;
        rt.block_on(db.signin(Root{
            username: "root",
            password: "root",
        }).into_future()).context("Signing in to database")?;
        rt.block_on(db.use_ns("HelixFlow").use_db("HelixFlow").into_future()).context("Selecting database namespace")?;
        
        Ok(Self {
            db,
            rt
        })

    }
}


impl StorageBackend<Thing> for SurrealDb {
    fn create(&self, task: &mut Task<Thing>) -> anyhow::Result<()> {
        let dbtask: Task<Thing> = self.rt.block_on(self.db.create("Tasks").content(Task::<Thing>{
            name: "Hardcoded".into(),
            id: None,
            description: None}
        ).into_future()).unwrap().context("Creating new record")?;
        task.id = dbtask.id;
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
            let backend = SurrealDb::connect().unwrap();
            let created = new_task.create(&backend);
            if let Err(e) = created {dbg!("Error: {}", e);};
            assert_eq!(new_task.name, "Test Task 1");
            assert_eq!(new_task.description, None);
            assert!(new_task.id != None);
        }
    }
}
