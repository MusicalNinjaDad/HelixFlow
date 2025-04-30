use anyhow::{Context, Ok};
use surrealdb::{Connection, Surreal, sql::Thing};

use tokio::runtime::Runtime;

use crate::task::{StorageBackend, Task};

/// A blocking instance of a SurrealDb
#[allow(dead_code)]
struct SurrealDb<C: Connection> {
    /// The async surrealdb Client
    db: Surreal<C>,

    /// A `current_thread` `tokio::Runtime`
    rt: Runtime,
}

impl<C: Connection> StorageBackend<Thing> for SurrealDb<C> {
    fn create(&self, task: &mut Task<Thing>) -> anyhow::Result<()> {
        let dbtask: Task<Thing> = self
            .rt
            .block_on(self.db.create("Tasks").content(task.clone()).into_future())
            .unwrap()
            .context("Creating new record")?;
        task.id = dbtask.id;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use surrealdb::engine::local::{Db, Mem};

    impl SurrealDb<Db> {
        fn connect() -> anyhow::Result<Self> {
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .context("Creating Tokio Runtime")?;

            let db = rt
                .block_on(Surreal::new::<Mem>(()).into_future())
                .context("Initialising database connection")?;
            rt.block_on(db.use_ns("HelixFlow").use_db("HelixFlow").into_future())
                .context("Selecting database namespace")?;

            Ok(Self { db, rt })
        }
    }

    #[test]
    fn test_new_task_id_updated() {
        {
            let mut new_task = Task {
                name: "Test Task 1".into(),
                id: None,
                description: None,
            };
            let backend = SurrealDb::connect().unwrap();
            new_task.create(&backend).unwrap(); // Unwrap to check we don't get any errors
            assert_eq!(new_task.name, "Test Task 1");
            assert_eq!(new_task.description, None);
            assert!(new_task.id != None);
        }
    }
    #[test]
    fn test_new_task_written_to_db() {
        {
            let mut new_task = Task {
                name: "Test Task 2".into(),
                id: None,
                description: None,
            };
            let backend = SurrealDb::connect().unwrap();
            new_task.create(&backend).unwrap(); // Unwrap to check we don't get any errors
            let id = new_task.id.unwrap();
            let stored_task: Task<Thing> = backend
                .rt
                .block_on(
                    backend
                        .db
                        .select((id.tb.clone(), id.id.to_raw()))
                        .into_future(),
                )
                .unwrap()
                .unwrap();
            assert_eq!(stored_task.name, new_task.name);
            assert_eq!(stored_task.description, new_task.description);
        }
    }
}
