use crate::memory::Message;

#[derive(sqlx::FromRow, Clone)]
pub struct MessageModelSql {
    pub id: String,
    pub thread_name: String,
    pub role: String,
    pub content: String,
}

pub struct GetMessageParams {
    pub thread_name: String,
}

pub struct CreateMessageBody {
    pub thread_name: String,
    pub role: String,
    pub content: String,
}

pub struct DeleteMessageParams {
    pub id: String,
}

impl CreateMessageBody {
    pub fn from_message(thread_name: &str, message: Message) -> Self {
        Self {
            thread_name: thread_name.to_string(),
            role: message.role().to_string(),
            content: message.content().unwrap_or(String::new()),
        }
    }
}
