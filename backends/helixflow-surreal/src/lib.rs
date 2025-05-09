//! Functionality to utilise a [`SurrealDb`](https://surrealdb.com) backend.

use std::{borrow::Cow, rc::Rc};

use anyhow::{Context, Ok, Result, anyhow};
use log::debug;
use surrealdb::{
    Connection, Surreal, Uuid,
    engine::{
        local::{Db, Mem},
        remote::ws::{Client, Ws},
    },
    opt::auth::Root,
    sql::{Id, Thing},
};

use serde::{Deserialize, Serialize};

use helixflow_core::task::Task;

#[derive(Debug, Serialize, Deserialize)]
struct SurrealTask {
    name: Cow<'static, str>,
    id: Thing,
    description: Option<Cow<'static, str>>,
}

impl TryFrom<SurrealTask> for Task {
    type Error = anyhow::Error;
    fn try_from(task: SurrealTask) -> Result<Task, Self::Error> {
        let id = match task.id.id {
            Id::Uuid(id) => Ok(id.into()),
            _ => Err(anyhow!("Not a valid UUID")),
        };
        Ok(Task {
            name: task.name,
            id: id?,
            description: task.description,
        })
    }
}

impl From<&Task> for SurrealTask {
    fn from(task: &Task) -> Self {
        SurrealTask {
            name: task.name.clone(),
            id: Thing::from(("Tasks", Id::Uuid(task.id.into()))),
            description: task.description.clone(),
        }
    }
}

pub mod blocking {
    use std::borrow::Cow;

    use super::*;
    use helixflow_core::task::{TaskCreationError, TaskResult, blocking::StorageBackend};
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

    /// SurrealDb returns a `Thing` as `id`.
    ///
    /// A `Thing` is a wierd SurrealDb Struct with a `tb` (= "table") and `id` field,
    /// both as owned `String`s :-x (!!)
    impl<C: Connection> StorageBackend for SurrealDb<C> {
        fn create(&self, task: &Task) -> anyhow::Result<Task> {
            dbg!(task);
            let dbtask: SurrealTask = self
                .rt
                .block_on(
                    self.db
                        .create("Tasks")
                        .content(SurrealTask::from(task))
                        .into_future(),
                )?
                .with_context(|| format!("Creating new record for {:#?} in SurrealDb", task))?;
            let checktask = dbtask.try_into()?;
            dbg!(&checktask);
            Ok(checktask)
        }
        fn get(&self, id: &Uuid) -> anyhow::Result<Task> {
            let dbtask: Option<SurrealTask> = self
                .rt
                .block_on(self.db.select(("Tasks", id.clone())).into_future())?;
            if let Some(task) = dbtask {
                Ok(task.try_into()?)
            } else {
                Err(anyhow!("Unknown task ID: {}", id))
            }
        }
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

    /// Can't run blocking on wasm as `runtime::Builder::enable_all()` needs `time` AND
    /// `block_on()` will not run either, as rt cannot be idle.
    #[cfg(test)]
    mod tests {
        use helixflow_core::task::blocking::CRUD;
        use std::str::FromStr;
        use wasm_bindgen_test::*;

        use super::*;

        wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

        #[test]
        fn test_new_task() {
            {
                let new_task = Task::new("Test Task 1", None);
                let backend = SurrealDb::create().unwrap();
                new_task.create(&backend).unwrap(); // Unwrap to check we don't get any errors
            }
        }

        #[test]
        fn test_new_task_written_to_db() {
            {
                let new_task = Task::new("Test Task 2", None);
                let backend = SurrealDb::create().unwrap();
                new_task.create(&backend).unwrap(); // Unwrap to check we don't get any errors
                let stored_task = backend.get(&new_task.id).unwrap();
                assert_eq!(stored_task, new_task);
            }
        }

        #[test]
        fn test_get_invalid_task() {
            {
                let backend = SurrealDb::create().unwrap();
                let id = Uuid::now_v7();
                let err = backend.get(&id).unwrap_err();
                assert_eq!(format!("{}", err), format!("Unknown task ID: {}", id));
            }
        }

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

pub mod non_blocking {
    use std::borrow::Cow;

    use super::*;
    use async_trait::async_trait;
    use helixflow_core::task::{TaskCreationError, TaskResult, non_blocking::StorageBackend};
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

    #[async_trait]
    impl<C: Connection> StorageBackend for SurrealDb<C> {
        async fn create(&self, task: &Task) -> anyhow::Result<Task> {
            dbg!(task);
            let dbtask: SurrealTask = self
                .db
                .create("Tasks")
                .content(SurrealTask::from(task))
                .await?
                .with_context(|| format!("Creating new record for {:#?} in SurrealDb", task))?;
            let checktask = dbtask.try_into()?;
            dbg!(&checktask);
            Ok(checktask)
        }
    }

    /// Instantiate an in-memory Db with `ns` & `db` = "HelixFlow".
    /// This is a blocking operation until the db is available.
    impl SurrealDb<Db> {
        pub async fn create() -> anyhow::Result<Self> {
            debug!("Initialising database");
            let db = Surreal::new::<Mem>(())
                .await
                .context("Initialising database")?;
            debug!("Selecting database namespace");
            db.use_ns("HelixFlow")
                .use_db("HelixFlow")
                .await
                .context("Selecting database namespace")?;
            debug!("Done connecting to database");
            Ok(Self { db })
        }
    }

    #[cfg(all(test, not(target_family = "wasm")))]
    mod tests {
        use helixflow_core::task::non_blocking::CRUD;
        use std::{str::FromStr, sync::Arc};

        use macro_rules_attribute::apply;
        use smol_macros::{Executor, test};

        use super::*;
        

        #[apply(test)]
        async fn test_new_task(exec: &Executor<'_>) {
            let new_task = Task::new("Test Task 1", None);
            let backend = Arc::new(SurrealDb::create().await.unwrap());
            let be = Arc::downgrade(&backend);
            exec.spawn(async move {
                let backend = be.upgrade().unwrap();
                new_task.create(backend.as_ref()).await
            })
            .await
            .unwrap();
        }

        // #[test]
        // fn test_new_task_written_to_db() {
        //     {
        //         let new_task = Task::new("Test Task 2", None);
        //         let backend = SurrealDb::create().unwrap();
        //         new_task.create(&backend).unwrap(); // Unwrap to check we don't get any errors
        //         let stored_task = backend.get(&new_task.id).unwrap();
        //         assert_eq!(stored_task, new_task);
        //     }
        // }

        // #[test]
        // fn test_get_invalid_task() {
        //     {
        //         let backend = SurrealDb::create().unwrap();
        //         let id = Uuid::now_v7();
        //         let err = backend.get(&id).unwrap_err();
        //         assert_eq!(format!("{}", err), format!("Unknown task ID: {}", id));
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
