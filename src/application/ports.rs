use async_trait::async_trait;

use crate::domain::todo::{NewTodo, Todo, UpdateTodo};

use super::errors::RepositoryError;

#[async_trait]
pub trait TodoRepository: Send + Sync {
    async fn list(&self) -> Result<Vec<Todo>, RepositoryError>;

    async fn find(&self, id: i32) -> Result<Option<Todo>, RepositoryError>;

    async fn create(&self, new_todo: NewTodo) -> Result<Todo, RepositoryError>;

    async fn update(
        &self,
        id: i32,
        update: UpdateTodo,
    ) -> Result<Option<Todo>, RepositoryError>;

    async fn delete(&self, id: i32) -> Result<bool, RepositoryError>;
}

