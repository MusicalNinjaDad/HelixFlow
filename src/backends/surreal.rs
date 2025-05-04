//! Functionality to utilise a [`SurrealDb`](https://surrealdb.com) backend.

use anyhow::{Context, Ok};
use surrealdb::{
    Connection, Surreal,
    engine::local::{Db, Mem},
    sql::Thing,
};

use crate::task::{StorageBackend, Task};

/// An instance of a SurrealDb ready to use as a `StorageBackend`
///
/// This requires some form of instantiation function, the exact specification of which will depend
/// on the type of `<C: Connection>` selected. See the unit tests for an example of instantiating
/// an in-memory Db.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct SurrealDb<C: Connection> {
    /// The instatiated Surreal Db `Connection`. This should be in an authenticated state with
    /// `namespace` & `database` already selected, so that functions such as `create()` can be
    /// called without further preamble.
    db: Surreal<C>,
}

/// SurrealDb returns a `Thing` as `id`.
///
/// A `Thing` is a wierd SurrealDb Struct with a `tb` (= "table") and `id` field,
/// both as owned `String`s :-x (!!)
impl<C: Connection> StorageBackend<Thing> for SurrealDb<C> {
    async fn create(&self, task: &mut Task<Thing>) -> anyhow::Result<()> {
        let dbtask: Task<Thing> = self
            .db
            .create("Tasks")
            .content(task.clone())
            .await
            .unwrap()
            .with_context(|| format!("Creating new record for {:#?} in SurrealDb", task))?;
        // TODO: Wrangle id (into a couple of `Cow`s?) so it can be passed directly to
        // `.select<O>(&self, resource: impl surrealdb::opt::IntoResource<O>)` without
        // ownership and conversion concerns. (Or only partially, to avoid taking time now, and take
        // the time to clone/convert only when needed, e.g. on the first attempt to select?)
        task.id = dbtask.id;
        Ok(())
    }
}

/// Instantiate an in-memory Db with `ns` & `db` = "HelixFlow"
impl SurrealDb<Db> {
    #[allow(dead_code)]
    pub async fn create() -> anyhow::Result<Self> {
        let db = Surreal::new::<Mem>(())
            .await
            .context("Initialising database")?;
        db.use_ns("HelixFlow")
            .use_db("HelixFlow")
            .await
            .context("Selecting database namespace")?;

        Ok(Self { db })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_new_task_id_updated() {
        {
            let mut new_task = Task {
                name: "Test Task 1".into(),
                id: None,
                description: None,
            };
            let backend = SurrealDb::create().await.unwrap();
            new_task.create(&backend).await.unwrap(); // Unwrap to check we don't get any errors
            assert_eq!(new_task.name, "Test Task 1");
            assert!(new_task.description.is_none());
            assert!(new_task.id.is_some());
        }
    }
    #[tokio::test]
    async fn test_new_task_written_to_db() {
        {
            let mut new_task = Task {
                name: "Test Task 2".into(),
                id: None,
                description: None,
            };
            let backend = SurrealDb::create().await.unwrap();
            new_task.create(&backend).await.unwrap(); // Unwrap to check we don't get any errors
            let id = new_task.id.unwrap();
            let stored_task: Task<Thing> = backend
                .db
                .select((id.tb.clone(), id.id.to_raw()))
                .await
                .unwrap()
                .unwrap();
            assert_eq!(stored_task.name, new_task.name);
            assert_eq!(stored_task.description, new_task.description);
        }
    }
}
