//! Functionality to utilise a [`SurrealDb`](https://surrealdb.com) backend.

use std::rc::Rc;

use anyhow::{Context, Ok, Result, anyhow};
use log::debug;
use surrealdb::{
    Connection, Surreal,
    engine::{
        local::{Db, Mem},
        remote::ws::{Client, Ws},
    },
    opt::auth::Root,
    sql::{Thing, Uuid, Id},
};

use serde::{Deserialize, Serialize};


use helixflow_core::task::Task;

pub mod blocking {
    use std::borrow::Cow;

    use super::*;
    use helixflow_core::task::{TaskResult, blocking::StorageBackend};
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

        /// A dedicated tokio runtime to allow for blocking operations
        rt: Rc<tokio::runtime::Runtime>,
    }

    #[derive(Debug, Serialize, Deserialize)]
    struct TaskNoId {
        name: Cow<'static, str>,
        id: Thing,
        description: Option<Cow<'static, str>>,
    }

    /// SurrealDb returns a `Thing` as `id`.
    ///
    /// A `Thing` is a wierd SurrealDb Struct with a `tb` (= "table") and `id` field,
    /// both as owned `String`s :-x (!!)
    impl<C: Connection> StorageBackend for SurrealDb<C> {
        fn create(&self, task: &Task) -> anyhow::Result<Task> {
            dbg!(task);
            let taskuuid: Uuid = task.id.into();
            let taskid: Id = taskuuid.into();
            let taskthing: Thing = Thing::from(("Tasks", taskid));
            let t = TaskNoId {name: task.name.clone(), id: taskthing, description: task.description.clone()};
            let dbtask: TaskNoId = self
                .rt
                .block_on(self.db.create("Tasks").content(t).into_future())?
                .with_context(|| format!("Creating new record for {:#?} in SurrealDb", task))?;
            // TODO: Wrangle id (into a couple of `Cow`s?) so it can be passed directly to
            // `.select<O>(&self, resource: impl surrealdb::opt::IntoResource<O>)` without
            // ownership and conversion concerns. (Or only partially, to avoid taking time now, and take
            // the time to clone/convert only when needed, e.g. on the first attempt to select?)
            let checktask = Task {
                name: dbtask.name,
                id: if let Id::Uuid(dbuuid) = dbtask.id.id {dbuuid.into()} else {Uuid::new_v7().into()},
                description: dbtask.description
            };
            dbg!(&checktask);
            Ok(checktask)
        }
        // fn get(&self, id: Thing) -> Result<Task<Thing>> {
        //     self.rt
        //         .block_on(
        //             self.db
        //                 .select((id.tb.clone(), id.id.to_raw()))
        //                 .into_future(),
        //         )?
        //         .ok_or_else(|| anyhow!("Invalid task ID: {}", id))
        // }
    }

    /// Instantiate an in-memory Db with `ns` & `db` = "HelixFlow".
    /// This is a blocking operation until the db is available.
    impl SurrealDb<Db> {
        pub fn create() -> anyhow::Result<Self> {
            debug!("Initialising tokio runtime");
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .context("Initialising dedicated tokio runtime for surreal in memory database.")?;
            debug!("Initialising database");
            let db = rt
                .block_on(Surreal::new::<Mem>(()).into_future())
                .context("Initialising database")?;
            debug!("Selecting database namespace");
            rt.block_on(db.use_ns("HelixFlow").use_db("HelixFlow").into_future())
                .context("Selecting database namespace")?;
            debug!("Stuffing the runtime in an Rc");
            let runtime = Rc::new(rt);
            debug!("Done connecting to database");
            Ok(Self { db, rt: runtime })
        }
    }

    impl SurrealDb<Client> {
        pub fn connect(address: &str) -> Result<Self> {
            debug!("Initialising tokio runtime");
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .context("Initialising dedicated tokio runtime for surreal in memory database.")?;
            debug!("Connecting to database");
            let db = rt
                .block_on(Surreal::new::<Ws>(address).into_future())
                .context("Connecting to database")?;
            debug!("Signing in to database");
            rt.block_on(
                db.signin(Root {
                    username: "root",
                    password: "root",
                })
                .into_future(),
            )
            .context("Signing in to database")?;
            debug!("Selecting database namespace");
            rt.block_on(db.use_ns("HelixFlow").use_db("HelixFlow").into_future())
                .context("Selecting database namespace")?;
            debug!("Stuffing the runtime in an Rc");
            let runtime = Rc::new(rt);
            debug!("Done connecting to database");
            Ok(Self { db, rt: runtime })
        }
    }

    #[cfg(test)]
    mod tests {
        use helixflow_core::task::blocking::TaskExt;
        use std::str::FromStr;

        use super::*;

        #[test]
        fn test_new_task_id_updated() {
            {
                let new_task = Task::new("Test Task 1", None);
                let backend = SurrealDb::create().unwrap();
                new_task.create(&backend).unwrap();
            }
        }

        // #[test]
        // fn test_new_task_written_to_db() {
        //     {
        //         let mut new_task = Task {
        //             name: "Test Task 2".into(),
        //             id: None,
        //             description: None,
        //         };
        //         let backend = SurrealDb::create().unwrap();
        //         new_task.create(&backend).unwrap(); // Unwrap to check we don't get any errors
        //         let id = new_task.id.unwrap();
        //         let stored_task: Task<Thing> = backend.get(id).unwrap();
        //         assert_eq!(stored_task.name, new_task.name);
        //         assert_eq!(stored_task.description, new_task.description);
        //     }
        // }

        // #[test]
        // fn test_get_invalid_task() {
        //     {
        //         let backend = SurrealDb::create().unwrap();
        //         let id = Thing::from_str("table:record").unwrap();
        //         let err = backend.get(id).unwrap_err();
        //         assert_eq!(format!("{}", err), "Invalid task ID: table:record");
        //     }
        // }

        // #[test]
        // fn test_new_task_written_to_external_db() {
        //     {
        //         let mut new_task = Task {
        //             name: "Test Task 2".into(),
        //             id: None,
        //             description: None,
        //         };
        //         let backend = SurrealDb::connect("localhost:8010").unwrap();
        //         new_task.create(&backend).unwrap(); // Unwrap to check we don't get any errors
        //         let id = new_task.id.unwrap();
        //         let stored_task: Task<Thing> = backend.get(id).unwrap();
        //         assert_eq!(stored_task.name, new_task.name);
        //         assert_eq!(stored_task.description, new_task.description);
        //     }
        // }
    }
}
