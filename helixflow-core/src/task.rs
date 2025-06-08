//! The fundamental `Task` building block and related functions.

use std::{
    any::Any,
    borrow::Cow,
    ops::{ControlFlow, FromResidual, Try},
};

use anyhow::anyhow;
use serde::{Deserialize, Serialize};
use uuid::{Uuid, uuid};

use crate::{
    HelixFlowError, HelixFlowItem, HelixFlowResult, Link, Linkable, Relate, Relationship, Store,
};

impl HelixFlowItem for Task {
    fn as_any(&self) -> &dyn Any {
        self
    }
}
impl HelixFlowItem for TaskList {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// A Task
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Task {
    // TODO check value of using a Cow here ... are we really saving when passing around - or would
    // it be better to have something that is `Copy`?
    pub name: Cow<'static, str>,
    pub id: Uuid,
    pub description: Option<Cow<'static, str>>,
}

impl Task {
    /// Create a new `Task` with valid `id`, suitable for usage as database key.
    ///
    /// `name` & `Some(description)` must be of the same type to avoid the need to specify the
    /// theoretical type of a `None` description.
    ///
    /// Even though `name` must be given, it may be an empty string `""` - semantically every
    /// Task has a name, even if this is blank, but not every Task has a description.
    pub fn new<S1>(name: S1, description: Option<S1>) -> Task
    where
        S1: Into<Cow<'static, str>>,
    {
        Task {
            name: name.into(),
            id: Uuid::now_v7(),
            description: description.map(|desc| desc.into()),
        }
    }
}

/// A list of tasks
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct TaskList {
    pub name: Cow<'static, str>,
    pub id: Uuid,
}

impl TaskList {
    /// Create a new `TaskList` with valid `id`, suitable for usage as database key.
    pub fn new<S>(name: S) -> TaskList
    where
        S: Into<Cow<'static, str>>,
    {
        TaskList {
            name: name.into(),
            id: Uuid::now_v7(),
        }
    }
}

#[derive(Debug)]
pub struct Contains<LEFT, RIGHT> {
    pub left: HelixFlowResult<LEFT>,
    pub sortorder: String,
    pub right: HelixFlowResult<RIGHT>,
}

impl Relationship for Contains<TaskList, Task> {
    type Left = TaskList;
    type Right = Task;
}

impl<LEFT, RIGHT> Try for Contains<LEFT, RIGHT>
where
    Contains<LEFT, RIGHT>: Relationship,
{
    type Output = Self; // Continue
    type Residual = Self; // Break
    fn branch(self) -> ControlFlow<Self::Residual, Self::Output> {
        if self.left.is_ok() && self.right.is_ok() {
            ControlFlow::Continue(self)
        } else {
            ControlFlow::Break(self)
        }
    }
    fn from_output(_output: Self::Output) -> Self {
        unimplemented!("Contains? should only be used in funtions returning a Result")
    }
}

impl<LEFT, RIGHT> FromResidual<Contains<LEFT, RIGHT>> for Contains<LEFT, RIGHT>
where
    Contains<LEFT, RIGHT>: Relationship,
{
    fn from_residual(_residual: Contains<LEFT, RIGHT>) -> Self {
        unimplemented!("Contains? should only be used in funtions returning a Result")
    }
}

impl<LEFT, RIGHT> FromResidual<Contains<LEFT, RIGHT>> for HelixFlowResult<()>
where
    Contains<LEFT, RIGHT>: Relationship,
    LEFT: HelixFlowItem,
    RIGHT: HelixFlowItem,
{
    fn from_residual(residual: Contains<LEFT, RIGHT>) -> Self {
        Err(HelixFlowError::RelationshipBetweenErrors {
            left: match residual.left {
                Ok(item) => Box::new(Ok(Box::new(item))),
                Err(e) => Box::new(Err(e)),
            },
            right: match residual.right {
                Ok(item) => Box::new(Ok(Box::new(item))),
                Err(e) => Box::new(Err(e)),
            },
        })
    }
}

impl<LEFT, RIGHT> Link for Contains<LEFT, RIGHT>
where
    Contains<LEFT, RIGHT>: Relationship,
    LEFT: HelixFlowItem,
    RIGHT: HelixFlowItem + Clone + PartialEq,
{
    fn create_linked_item<B: Relate<Contains<LEFT, RIGHT>>>(
        self,
        backend: &B,
    ) -> HelixFlowResult<()> {
        let valid_relationship = self?;
        let created = backend.create_linked_item(&valid_relationship)?;
        let _tasklist_ok = created.left?;
        let expected = valid_relationship.right?;
        match created.right {
            Ok(task) if task == expected => Ok(()),
            Ok(_) => Err(HelixFlowError::Mismatch {
                expected: Box::new(expected.clone()),
                actual: Box::new(created.right?.clone()),
            }),
            Err(e) => Err(e),
        }
    }
}

impl<LEFT, RIGHT> Linkable<Contains<LEFT, RIGHT>> for LEFT
where
    Contains<LEFT, RIGHT>: Relationship<Left = LEFT, Right = RIGHT>,
    LEFT: HelixFlowItem + Clone + PartialEq,
    RIGHT: HelixFlowItem + Clone + PartialEq,
{
    fn link(&self, task: &RIGHT) -> Contains<LEFT, RIGHT> {
        Contains {
            left: Ok(self.clone()),
            sortorder: "a".into(),
            right: Ok(task.clone()),
        }
    }
    fn get_linked_items<B>(
        &self,
        backend: &B,
    ) -> HelixFlowResult<impl Iterator<Item = Contains<LEFT, RIGHT>>>
    where
        B: Relate<Contains<LEFT, RIGHT>>,
    {
        backend.get_linked_items(self)
    }
}

#[derive(Clone, Copy)]
pub struct TestBackend;

impl Store<Task> for TestBackend {
    fn create(&self, task: &Task) -> HelixFlowResult<Task> {
        match task.name {
            Cow::Borrowed("FAIL") => Err(anyhow!("Failed to create task").into()),
            Cow::Borrowed("MISMATCH") => Ok(Task::new(task.name.clone(), task.description.clone())),
            _ => Ok(task.clone()),
        }
    }

    fn get(&self, id: &Uuid) -> HelixFlowResult<Task> {
        match id.to_string().as_str() {
            "0196b4c9-8447-7959-ae1f-72c7c8a3dd36" => Ok(Task {
                name: "Task 1".into(),
                id: *id,
                description: None,
            }),
            "0196ca5f-d934-7ec8-b042-ae37b94b8432" => Ok(Task {
                name: "Task 2".into(),
                id: *id,
                description: None,
            }),
            _ => Err(HelixFlowError::NotFound {
                itemtype: "Task".into(),
                id: *id,
            }),
        }
    }
}

impl Store<TaskList> for TestBackend {
    fn create(&self, _item: &TaskList) -> HelixFlowResult<TaskList> {
        todo!()
    }
    fn get(&self, id: &Uuid) -> HelixFlowResult<TaskList> {
        match id.to_string().as_str() {
            "0196fe23-7c01-7d6b-9e09-5968eb370549" => Ok(TaskList {
                name: "Test TaskList 1".into(),
                id: *id,
            }),
            _ => Err(HelixFlowError::NotFound {
                itemtype: "Tasklist".into(),
                id: *id,
            }),
        }
    }
}

impl Relate<Contains<TaskList, Task>> for TestBackend {
    fn create_linked_item(
        &self,
        link: &Contains<TaskList, Task>,
    ) -> HelixFlowResult<Contains<TaskList, Task>> {
        let tasklist = link.left.as_ref().unwrap().clone();
        match tasklist.id.to_string().as_str() {
            "0196fe23-7c01-7d6b-9e09-5968eb370549" => Ok(Contains {
                left: Ok(tasklist),
                sortorder: link.sortorder.clone(),
                right: self.create(link.right.as_ref().unwrap()),
            }),
            _ => Err(HelixFlowError::NotFound {
                itemtype: "Tasklist".into(),
                id: tasklist.id,
            }),
        }
    }
    fn get_linked_items(
        &self,
        left: &TaskList,
    ) -> HelixFlowResult<impl Iterator<Item = Contains<TaskList, Task>>> {
        match left.id.to_string().as_str() {
            "0196fe23-7c01-7d6b-9e09-5968eb370549" => {
                let tasks = vec![
                    Task {
                        name: "Task 1".into(),
                        id: uuid!("0196b4c9-8447-7959-ae1f-72c7c8a3dd36"),
                        description: None,
                    },
                    Task {
                        name: "Task 2".into(),
                        id: uuid!("0196ca5f-d934-7ec8-b042-ae37b94b8432"),
                        description: None,
                    },
                ];
                Ok(tasks.into_iter().map(|task| left.link(&task)))
            }
            _ => Err(HelixFlowError::NotFound {
                itemtype: "Tasklist".into(),
                id: left.id,
            }),
        }
    }
}

#[cfg(test)]
#[coverage(off)]
mod tests {
    use crate::CRUD;

    use super::*;
    use std::assert_matches::assert_matches;

    #[test]
    fn try_contains_oks() -> HelixFlowResult<()> {
        let tasklist = TaskList::new("tasklist");
        let task = Task::new("task", None);
        let contains = Contains {
            left: Ok(tasklist.clone()),
            sortorder: "a".into(),
            right: Ok(task.clone()),
        };
        let contains2 = Contains {
            left: Ok(tasklist.clone()),
            sortorder: "a".into(),
            right: Ok(task.clone()),
        };
        let contains = contains?;
        assert_eq!(contains.left.unwrap(), contains2.left.unwrap());
        assert_eq!(contains.right.unwrap(), contains2.right.unwrap());
        Ok(())
    }

    #[test]
    fn try_contains_err_left() {
        let task = Task::new("task", None);
        let contains: Contains<TaskList, Task> = Contains {
            left: Err(HelixFlowError::NotFound {
                itemtype: "Tasklist".into(),
                id: uuid!("0196b4c9-8447-7959-ae1f-72c7c8a3dd36"),
            }),
            sortorder: "try_contains_err_left".into(),
            right: Ok(task.clone()),
        };
        fn is_valid(relationship: Contains<TaskList, Task>) -> HelixFlowResult<()> {
            relationship?;
            Ok(())
        }
        let err = is_valid(contains).unwrap_err();
        assert_matches!(
            err,
            HelixFlowError::RelationshipBetweenErrors { left, right }
            if matches!(
                left.as_ref(),
                Err(boxed_err) if matches!(
                    boxed_err,
                    HelixFlowError::NotFound {itemtype, id}
                    if itemtype == "Tasklist" && id == &uuid!("0196b4c9-8447-7959-ae1f-72c7c8a3dd36")
                )
            ) && matches!(
                right.as_ref(),
                Ok(boxed_task) if boxed_task.as_any().downcast_ref::<Task>() == Some(&task)
            )
        )
    }

    #[test]
    fn test_new_task() {
        let new_task = Task::new("Test Task", None);
        assert_eq!(new_task.name, "Test Task");
        assert!(new_task.description.is_none());
        assert!(!new_task.id.is_nil());
        assert_eq!(new_task.id.get_version(), Some(uuid::Version::SortRand));
    }

    #[test]
    fn test_new_task_blank() {
        let new_task = Task::new("", None);
        assert_eq!(new_task.name, "");
        assert!(new_task.description.is_none());
        assert!(!new_task.id.is_nil());
        assert_eq!(new_task.id.get_version(), Some(uuid::Version::SortRand));
    }

    #[test]
    fn test_create_task() {
        let new_task = Task::new("Test Task 1", None);
        let backend = TestBackend;
        new_task.create(&backend).unwrap();
    }

    #[test]
    fn test_failed_to_create_task() {
        let new_task = Task::new("FAIL", None);
        let backend = TestBackend;
        let err = new_task.create(&backend).unwrap_err();
        assert_matches!(err, HelixFlowError::BackendError(_))
    }

    #[test]
    fn test_mismatched_task_created() {
        let new_task = Task::new("MISMATCH", None);
        let backend = TestBackend;
        let err = new_task.create(&backend).unwrap_err();
        assert_matches!(
            err,
            HelixFlowError::Mismatch {
                expected: _,
                actual: _
            }
        )
    }

    #[test]
    fn test_get_task() {
        let backend = TestBackend;
        let id = uuid!("0196b4c9-8447-7959-ae1f-72c7c8a3dd36");
        let task = Task::get(&backend, &id).unwrap();
        assert_eq!(
            task,
            Task {
                name: "Task 1".into(),
                id,
                description: None
            }
        );
    }

    #[test]
    fn test_get_invalid_task() {
        let backend = TestBackend;
        let id = uuid!("0196b4c9-8447-78db-ae8a-be68a8095aa2");
        let err = Task::get(&backend, &id).unwrap_err();
        assert_eq!(
            format!("{}", &err),
            "404 No Task found with id 0196b4c9-8447-78db-ae8a-be68a8095aa2"
        );
        assert_matches!(
                err,
                HelixFlowError::NotFound { itemtype, id }
                if itemtype == "Task" && id == uuid!("0196b4c9-8447-78db-ae8a-be68a8095aa2"));
    }

    #[test]
    fn get_tasks_in_tasklist() {
        let backend = TestBackend;
        let backlog = TaskList {
            name: "Backlog".into(),
            id: uuid!("0196fe23-7c01-7d6b-9e09-5968eb370549"),
        };
        let task1 = Task {
            name: "Task 1".into(),
            id: uuid!("0196b4c9-8447-7959-ae1f-72c7c8a3dd36"),
            description: None,
        };
        let task2 = Task {
            name: "Task 2".into(),
            id: uuid!("0196ca5f-d934-7ec8-b042-ae37b94b8432"),
            description: None,
        };
        let tasks: Vec<Contains<TaskList, Task>> =
            backlog.get_linked_items(&backend).unwrap().collect();
        assert_eq!(
            tasks
                .into_iter()
                .map(|link| link.right.unwrap())
                .collect::<Vec<Task>>(),
            vec![task1, task2]
        );
    }

    #[test]
    fn create_task_in_tasklist() {
        use crate::task::{Contains, Link};
        let backend = TestBackend;
        let backlog = TaskList {
            name: "Backlog".into(),
            id: uuid!("0196fe23-7c01-7d6b-9e09-5968eb370549"),
        };
        let task3 = Task::new("Test task 3", None);
        let relationship: Contains<TaskList, Task> = backlog.link(&task3);
        relationship.create_linked_item(&backend).unwrap();
    }

    #[test]
    fn create_task_in_tasklist_mismatch() {
        use crate::task::{Contains, Link};
        let backend = TestBackend;
        let backlog = TaskList {
            name: "Backlog".into(),
            id: uuid!("0196fe23-7c01-7d6b-9e09-5968eb370549"),
        };
        let task3 = Task::new("MISMATCH", None);
        let relationship: Contains<TaskList, Task> = backlog.link(&task3);
        let mismatch = relationship.create_linked_item(&backend).unwrap_err();
        assert_matches!(
            mismatch,
            HelixFlowError::Mismatch { expected, actual: _ }
            if expected.as_ref().as_any().downcast_ref::<Task>() == Some(&task3)
        )
    }
}
