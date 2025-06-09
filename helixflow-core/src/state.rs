use std::{any::Any};

use uuid::Uuid;

use crate::{HelixFlowItem, task::TaskList};

/// The UI State. Uses builder pattern...
#[derive(Debug, Default, PartialEq, Clone)]
#[non_exhaustive]
pub struct State {
    visible_backlog: Option<Uuid>,
    id: Uuid,
}

impl HelixFlowItem for State {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl State {
    pub fn new(id: &Uuid) -> Self
    {
        State {
            id: *id,
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
