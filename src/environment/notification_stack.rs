use super::dispatch::EnvNotification;

use std::{collections::VecDeque, sync::Arc};

use tokio::sync::RwLock;
use uuid::Uuid;

pub type RefCountedNotificationStack = Arc<RwLock<NotificationStack>>;

#[derive(Debug)]
pub struct NotificationStack(VecDeque<EnvNotification>);

// impl Clone for NotificationStack {
//     fn clone(&self) -> Self {
//         Arc::clone(&self.0).into()
//     }
// }

impl From<VecDeque<EnvNotification>> for NotificationStack {
    fn from(value: VecDeque<EnvNotification>) -> Self {
        Self(value)
    }
}

impl Into<VecDeque<EnvNotification>> for NotificationStack {
    fn into(self) -> VecDeque<EnvNotification> {
        self.0
    }
}

impl NotificationStack {
    pub fn len(&self) -> usize {
        self.0.len()
    }
    /// Get the least recent notification on the stack
    pub fn pop_front(&mut self) -> Option<EnvNotification> {
        self.0.pop_front()
    }

    /// Get the most recent notification on the stack
    pub fn pop_back(&mut self) -> Option<EnvNotification> {
        self.0.pop_back()
    }

    /// Removes notifications with given agent id from stack and returns them as VecDeque
    #[tracing::instrument(name = "Take notifications by agent id")]
    pub fn take_by_agent(&mut self, agent_id: &str) -> Option<VecDeque<EnvNotification>> {
        let (matching, remaining): (VecDeque<EnvNotification>, VecDeque<EnvNotification>) = self
            .0
            .drain(..)
            .partition(|noti| noti.agent_id() == Some(agent_id));
        self.0 = remaining;
        if matching.len() != 0 {
            return Some(matching.into());
        }
        None
    }

    /// Removes the most recent notification with the given ticket number from the stack
    #[tracing::instrument(name = "Take notification by ticket number")]
    pub fn take_by_ticket(&mut self, ticket: Uuid) -> Option<EnvNotification> {
        if let Some(index) = self
            .0
            .iter_mut()
            .position(|noti| noti.ticket_number() == Some(ticket))
        {
            self.0.remove(index)
        } else {
            None
        }
    }

    /// Pushes given notification to the front
    pub(crate) async fn push(&mut self, noti: EnvNotification) {
        match noti {
            EnvNotification::AgentStateUpdate { ref agent_id, .. } => {
                let outer_id = agent_id;
                self.0.retain(|noti| match noti {
                    EnvNotification::AgentStateUpdate { agent_id, .. } => &agent_id != &outer_id,
                    _ => true,
                });
            }
            _ => {}
        }
        self.0.push_front(noti);
    }
}
