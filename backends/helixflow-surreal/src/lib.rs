#![feature(assert_matches)]
#![feature(coverage_attribute)]
#![feature(let_chains)]
//! Functionality to utilise a [`SurrealDb`](https://surrealdb.com) backend.

use std::{borrow::Cow, path::PathBuf, rc::Rc};

use anyhow::Context;
use log::debug;
use serde::{Deserialize, Serialize};
use surrealdb::{
    Connection, Surreal, Uuid,
    engine::local::{Db, Mem},
    error::Api,
    sql::{Id, Thing},
};

use helixflow_core::{
    HelixFlowError, HelixFlowResult,
    state::State,
    task::{Task, TaskList},
};

#[derive(Debug, Serialize, Deserialize)]
/// SurrealDb returns a `Thing` as `id`.
///
/// A `Thing` is a wierd SurrealDb Struct with a `tb` (= "table") and `id` field,
/// both as owned `String`s :-x (!!)
struct SurrealTask {
    name: Cow<'static, str>,
    id: Thing,
    description: Option<Cow<'static, str>>,
}

impl TryFrom<SurrealTask> for Task {
    type Error = HelixFlowError;
    fn try_from(task: SurrealTask) -> HelixFlowResult<Task> {
        let id = match task.id.id {
            Id::Uuid(id) => Ok(id.into()),
            _ => Err(HelixFlowError::InvalidID {
                id: task.id.id.to_string(),
            }),
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

#[derive(Debug, Serialize, Deserialize)]
/// SurrealDb returns a `Thing` as `id`.
///
/// A `Thing` is a wierd SurrealDb Struct with a `tb` (= "table") and `id` field,
/// both as owned `String`s :-x (!!)
struct SurrealTaskList {
    name: Cow<'static, str>,
    id: Thing,
}

impl TryFrom<SurrealTaskList> for TaskList {
    type Error = HelixFlowError;
    fn try_from(tasklist: SurrealTaskList) -> HelixFlowResult<TaskList> {
        let id = match tasklist.id.id {
            Id::Uuid(id) => Ok(id.into()),
            _ => Err(HelixFlowError::InvalidID {
                id: tasklist.id.id.to_string(),
            }),
        };
        Ok(TaskList {
            name: tasklist.name,
            id: id?,
        })
    }
}

impl From<&TaskList> for SurrealTaskList {
    fn from(tasklist: &TaskList) -> Self {
        SurrealTaskList {
            name: tasklist.name.clone(),
            id: Thing::from(("Tasklists", Id::Uuid(tasklist.id.into()))),
        }
    }
}
#[derive(Serialize, Deserialize, Debug, Clone)]
struct Link {
    r#in: Thing,
    out: Thing,
}

use helixflow_core::{Relate, Store, task::Contains};
/// An instance of a SurrealDb ready to use as a `StorageBackend`
///
/// This requires some form of instantiation function, the exact specification of which will depend
/// on the type of `<C: Connection>` selected. See the unit tests for an example of instantiating
/// an in-memory Db.
#[allow(dead_code)]
#[derive(Debug)]
pub struct SurrealDb<C: Connection> {
    /// The instatiated Surreal Db `Connection`. This should be in an authenticated state with
    /// `namespace` & `database` already selected, so that functions such as `create()` can be
    /// called without further preamble.
    db: Surreal<C>,

    /// A dedicated tokio runtime to allow for blocking operations
    rt: Rc<tokio::runtime::Runtime>,

    /// A file where the data will be persisted
    file: Option<PathBuf>,
}

impl<C: Connection> Store<Task> for SurrealDb<C> {
    fn create(&self, task: &Task) -> HelixFlowResult<Task> {
        dbg!(task);
        let dbtask: SurrealTask = self
            .rt
            .block_on(
                self.db
                    .create("Tasks")
                    .content(SurrealTask::from(task))
                    .into_future(),
            )
            .map_err(anyhow::Error::from)?
            .with_context(|| format!("Creating new record for {:#?} in SurrealDb", task))?;
        let checktask = dbtask.try_into()?;
        dbg!(&checktask);
        Ok(checktask)
    }

    fn get(&self, id: &Uuid) -> HelixFlowResult<Task> {
        let dbtask: Option<SurrealTask> = self
            .rt
            .block_on(self.db.select(("Tasks", *id)).into_future())
            .map_err(anyhow::Error::from)?;
        if let Some(task) = dbtask {
            Ok(task.try_into()?)
        } else {
            Err(HelixFlowError::NotFound {
                itemtype: "Task".into(),
                id: *id,
            })
        }
    }
}

impl<C: Connection> Store<TaskList> for SurrealDb<C> {
    fn create(&self, tasklist: &TaskList) -> HelixFlowResult<TaskList> {
        dbg!(tasklist);
        let dbtasklist: SurrealTaskList = self
            .rt
            .block_on(
                self.db
                    .create("Tasklists")
                    .content(SurrealTaskList::from(tasklist))
                    .into_future(),
            )
            .map_err(anyhow::Error::from)?
            .with_context(|| format!("Creating new record for {:#?} in SurrealDb", tasklist))?;
        let check_tasklist = dbtasklist.try_into()?;
        dbg!(&check_tasklist);
        Ok(check_tasklist)
    }

    fn get(&self, id: &Uuid) -> HelixFlowResult<TaskList> {
        let db_tasklist: Option<SurrealTaskList> = self
            .rt
            .block_on(self.db.select(("Tasklists", *id)).into_future())
            .map_err(anyhow::Error::from)?;
        if let Some(tasklist) = db_tasklist {
            Ok(tasklist.try_into()?)
        } else {
            Err(HelixFlowError::NotFound {
                itemtype: "TaskList".into(),
                id: *id,
            })
        }
    }
}

impl<C: Connection> Relate<Contains<TaskList, Task>> for SurrealDb<C> {
    fn create_linked_item(
        &self,
        link: &Contains<TaskList, Task>,
    ) -> HelixFlowResult<Contains<TaskList, Task>> {
        // TODO make this atomic
        let tasklist = link.left.as_ref().unwrap();
        // TODO - RelBetwErrs (or impl Try for &Contains ...)
        let task = link.right.as_ref().unwrap();
        dbg!(tasklist);
        let db_tasklist = self.get(&tasklist.id)?;
        let db_task = self.create(task)?;
        let confirmed_link: Vec<Link> = self
            .rt
            .block_on(
                self.db
                    .insert("contains")
                    .relation(Link {
                        r#in: SurrealTaskList::from(&db_tasklist).id,
                        out: SurrealTask::from(&db_task).id,
                    })
                    .into_future(),
            )
            .map_err(anyhow::Error::from)?;
        dbg!(confirmed_link);
        Ok(Contains {
            left: Ok(db_tasklist),
            sortorder: "a".into(),
            right: Ok(db_task),
        })
    }
    fn get_linked_items(
        &self,
        left: &TaskList,
    ) -> HelixFlowResult<impl Iterator<Item = Contains<TaskList, Task>>> {
        let tasklist: SurrealTaskList = left.into();
        dbg!(&tasklist);
        let mut tasks = self
            .rt
            .block_on(
                self.db
                    .query("SELECT ->contains->Tasks.* AS tasks FROM $tl")
                    .bind(("tl", tasklist.id))
                    .into_future(),
            )
            .map_err(anyhow::Error::from)?;
        dbg!(&tasks);
        let tasks: Vec<Vec<SurrealTask>> = tasks.take("tasks").map_err(anyhow::Error::from)?;
        dbg!(&tasks);
        let relationships = tasks
            .into_iter()
            .next()
            .unwrap()
            .into_iter()
            .map(|task| Contains {
                left: Ok(left.clone()),
                sortorder: "a".into(),
                right: task.try_into(),
            });
        Ok(relationships)
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct SurrealState {
    visible_backlog: Option<Uuid>,
    id: Thing,
}

impl TryFrom<SurrealState> for State {
    type Error = HelixFlowError;
    fn try_from(state: SurrealState) -> HelixFlowResult<State> {
        let id = match state.id.id {
            Id::Uuid(id) => Ok(id.into()),
            _ => Err(HelixFlowError::InvalidID {
                id: state.id.id.to_string(),
            }),
        };
        let mut stored_state = State::new(&id?);
        stored_state.visible_backlog(&TaskList {
            name: "".into(),
            id: state.visible_backlog.unwrap(),
        });
        Ok(stored_state)
    }
}

impl From<&State> for SurrealState {
    fn from(state: &State) -> Self {
        SurrealState {
            visible_backlog: *state.visible_backlog_id(),
            id: Thing::from(("State", Id::Uuid(state.id.into()))),
        }
    }
}

impl<C: Connection> Store<State> for SurrealDb<C> {
    fn create(&self, state: &State) -> HelixFlowResult<State> {
        dbg!(state);
        let dbstate: SurrealState = self
            .rt
            .block_on(
                self.db
                    .create("State")
                    .content(SurrealState::from(state))
                    .into_future(),
            )
            .map_err(anyhow::Error::from)?
            .with_context(|| format!("Creating new record for {:#?} in SurrealDb", state))?;
        let checkstate = dbstate.try_into()?;
        dbg!(&checkstate);
        Ok(checkstate)
    }

    fn get(&self, id: &Uuid) -> HelixFlowResult<State> {
        let dbstate: Option<SurrealState> = self
            .rt
            .block_on(self.db.select(("State", *id)).into_future())
            .map_err(anyhow::Error::from)?;
        if let Some(state) = dbstate {
            Ok(state.try_into()?)
        } else {
            Err(HelixFlowError::NotFound {
                itemtype: "State".into(),
                id: *id,
            })
        }
    }
}

impl SurrealDb<Db> {
    /// Instantiate an local Db, with data saved in `Some(file)` on drop,
    /// or simply held in memory (`None`).
    ///
    /// Note:
    /// - `ns` & `db` = "HelixFlow"
    /// - This is a blocking operation until the db is available.
    pub fn new(file: Option<PathBuf>) -> anyhow::Result<Self> {
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
        if let Some(file) = &file {
            let imported = rt.block_on(db.import(file).into_future());

            if let Err(e) = &imported
                && let surrealdb::Error::Api(Api::FileOpen { error, path }) = e
                && error.kind() == std::io::ErrorKind::NotFound
                && path == file
            {
                Ok(())
            } else {
                imported
            }
            .context(format!("Importing {:#?}", file))?
        }
        debug!("Stuffing the runtime in an Rc");
        let runtime = Rc::new(rt);
        debug!("Done connecting to database");
        Ok(Self {
            db,
            rt: runtime,
            file,
        })
    }
}

impl<C> Drop for SurrealDb<C>
where
    C: Connection,
{
    fn drop(&mut self) {
        if let Some(file) = &self.file {
            println!("Saving to {:#?}", file);
            self.rt
                .block_on(self.db.export(file).into_future())
                .unwrap()
            // TODO - handle errors nicely
        }
    }
}

// Can't run blocking on wasm as `runtime::Builder::enable_all()` needs `time` AND
// `block_on()` will not run either, as rt cannot be idle.
#[cfg(test)]
#[coverage(off)]
mod tests {
    use std::assert_matches::assert_matches;

    use super::*;

    use rstest::*;

    use tempfile::{NamedTempFile, TempPath};

    #[derive(Clone, Copy, Debug)]
    enum BackendKind {
        Mem,
        File,
    }

    struct Backend {
        _file_destructor: Option<TempPath>,
        backend: SurrealDb<Db>,
    }

    impl From<BackendKind> for Backend {
        fn from(kind: BackendKind) -> Self {
            match kind {
                BackendKind::Mem => Backend {
                    _file_destructor: None,
                    backend: SurrealDb::new(None).unwrap(),
                },
                BackendKind::File => {
                    let tmpfile = NamedTempFile::new().unwrap();
                    let location = tmpfile.path().into();
                    let tmppath = tmpfile.into_temp_path();
                    std::fs::remove_file(&location).unwrap();
                    Backend {
                        _file_destructor: Some(tmppath),
                        backend: SurrealDb::new(Some(location)).unwrap(),
                    }
                }
            }
        }
    }

    #[rstest]
    #[case(BackendKind::Mem)]
    #[case(BackendKind::File)]
    fn test_new_task(#[case] kind: BackendKind) {
        let Backend {
            _file_destructor,
            backend,
        } = kind.into();
        let new_task = Task::new("Test Task 1", None);
        backend.create(&new_task).unwrap();
    }

    #[rstest]
    #[case(BackendKind::Mem)]
    #[case(BackendKind::File)]
    fn test_new_task_written_to_db(#[case] kind: BackendKind) {
        let Backend {
            _file_destructor,
            backend,
        } = kind.into();
        let new_task = Task::new("Test Task 2", None);
        backend.create(&new_task).unwrap();
        let stored_task: Task = backend.get(&new_task.id).unwrap();
        assert_eq!(stored_task, new_task);
    }

    #[rstest]
    #[case(BackendKind::Mem)]
    #[case(BackendKind::File)]
    fn test_get_not_found(#[case] kind: BackendKind) {
        let Backend {
            _file_destructor,
            backend,
        } = kind.into();
        let id = Uuid::now_v7();
        let res: HelixFlowResult<Task> = backend.get(&id);
        let err = res.unwrap_err();
        assert_matches!(
            err,
            HelixFlowError::NotFound { itemtype, id: errid }
            if itemtype == "Task" && errid == id
        );
    }

    #[test]
    fn test_save_and_load() {
        let location = NamedTempFile::new().unwrap();
        let new_task = Task::new("Test Task 1", None);

        let file = location.path().to_path_buf();
        {
            let backend1 = SurrealDb::new(Some(file)).unwrap();
            backend1.create(&new_task).unwrap();
        } // backend1 destructor should store task in file

        let file = location.path().to_path_buf();
        let backend2 = SurrealDb::new(Some(file)).unwrap();
        let stored_task: Task = backend2.get(&new_task.id).unwrap();
        assert_eq!(stored_task, new_task);
    }
}
