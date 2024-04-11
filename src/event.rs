use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Event {
    pub id: String,
    pub title: String,
}

impl Event {
    pub fn new(id: String, title: String) -> Self {
        Event { id, title }
    }
}
