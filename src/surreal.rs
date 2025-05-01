//! Functionality to utilise a [`SurrealDb`](https://surrealdb.com) backend.

use anyhow::{Context, Ok};
use surrealdb::{Connection, Surreal, sql::Thing};

use tokio::runtime::Runtime;

use crate::task::{StorageBackend, Task};

/// An instance of a SurrealDb, which can be used in a blocking manner.
///
/// This requires some form of instantiation function, the exact specification of which will depend
/// on the type of `<C: Connection>` selected. See the unit tests for an example of instantiating
/// an in-memory Db.
///
/// Blocking calls to the Db can be made with
/// `self.rt.block_on(self.db.somefunction().into_future())`
#[allow(dead_code)]
struct SurrealDb<C: Connection> {
    /// The instatiated Surreal Db `Connection`. This should be in an authenticated state with
    /// `namespace` & `database` already selected, so that functions such as `create()` can be
    /// called without further preamble.
    db: Surreal<C>,

    /// A `current_thread` `tokio::Runtime` for the same thread as the Connection.
    rt: Runtime,
}

/// SurrealDb returns a `Thing` as `id`.
///
/// A `Thing` is a wierd SurrealDb Struct with a `tb` (= "table") and `id` field,
/// both as owned `String`s :-x (!!)
impl<C: Connection> StorageBackend<Thing> for SurrealDb<C> {
    fn create(&self, task: &mut Task<Thing>) -> anyhow::Result<()> {
        let dbtask: Task<Thing> = self
            .rt
            .block_on(self.db.create("Tasks").content(task.clone()).into_future())
            .unwrap()
            .with_context(|| format!("Creating new record for {:#?} in SurrealDb", task))?;
        // TODO: Wrangle id (into a couple of `Cow`s?) so it can be passed directly to
        // `.selectselect<O>(&self, resource: impl surrealdb::opt::IntoResource<O>)` without
        // ownership and conversion concerns. (Or only partially, to avoid taking time now, and take
        // the time to clone/convert only when needed, e.g. on the first attempt to select?)
        task.id = dbtask.id;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use surrealdb::engine::local::{Db, Mem};

    /// Instantiate an in-memory Db with `ns` & `db` = "HelixFlow"
    impl SurrealDb<Db> {
        fn create() -> anyhow::Result<Self> {
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .context("Creating Tokio Runtime")?;

            let db = rt
                .block_on(Surreal::new::<Mem>(()).into_future())
                .context("Initialising database")?;
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
            let backend = SurrealDb::create().unwrap();
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
            let backend = SurrealDb::create().unwrap();
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
