use std::{any::Any, borrow::Cow};

use uuid::Uuid;

use crate::{HelixFlowItem, task::TaskList};

/// The UI State. Uses builder pattern...
#[derive(Debug, Default, PartialEq, Clone)]
#[non_exhaustive]
pub struct State {
    visible_backlog: Option<Uuid>,
    id: Cow<'static, str>,
}

impl HelixFlowItem for State {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl State {
    pub fn new<ID>(id: ID) -> Self
    where
        ID: Into<Cow<'static, str>>,
    {
        State {
            id: id.into(),
            ..Default::default()
        }
    }

    pub fn visible_backlog(&mut self, backlog: &TaskList) {
        self.visible_backlog = Some(backlog.id);
    }

    pub fn visible_backlog_id(&self) -> &Option<Uuid> {
        &self.visible_backlog
    }
}
