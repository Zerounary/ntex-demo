use chrono::{DateTime, Utc};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Todo {
    pub id: i32,
    pub title: String,
    pub completed: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Clone, Debug)]
pub struct NewTodo {
    pub title: String,
    pub completed: bool,
}

#[derive(Clone, Debug, Default)]
pub struct UpdateTodo {
    pub title: Option<String>,
    pub completed: Option<bool>,
}

