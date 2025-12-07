use crate::domain::todo::{NewTodo, Todo, UpdateTodo};

use super::errors::UsecaseError;
use super::ports::TodoRepository;

pub struct TodoUseCase<R> {
    repository: R,
}

impl<R> TodoUseCase<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }
}

impl<R> TodoUseCase<R>
where
    R: TodoRepository,
{
    pub async fn list(&self) -> Result<Vec<Todo>, UsecaseError> {
        let todos = self.repository.list().await?;
        Ok(todos)
    }

    pub async fn get(&self, id: i32) -> Result<Todo, UsecaseError> {
        self.repository
            .find(id)
            .await?
            .ok_or(UsecaseError::NotFound("Todo"))
    }

    pub async fn create(&self, mut new_todo: NewTodo) -> Result<Todo, UsecaseError> {
        new_todo.title = Self::validate_title(&new_todo.title)?;
        let todo = self.repository.create(new_todo).await?;
        Ok(todo)
    }

    pub async fn update(
        &self,
        id: i32,
        mut update: UpdateTodo,
    ) -> Result<Todo, UsecaseError> {
        if update.title.is_none() && update.completed.is_none() {
            return Err(UsecaseError::Validation(
                "Provide at least one field to update.".into(),
            ));
        }

        if let Some(title) = update.title {
            update.title = Some(Self::validate_title(&title)?);
        }

        self.repository
            .update(id, update)
            .await?
            .ok_or(UsecaseError::NotFound("Todo"))
    }

    pub async fn delete(&self, id: i32) -> Result<(), UsecaseError> {
        if self.repository.delete(id).await? {
            Ok(())
        } else {
            Err(UsecaseError::NotFound("Todo"))
        }
    }

    fn validate_title(input: &str) -> Result<String, UsecaseError> {
        let trimmed = input.trim();
        if trimmed.is_empty() {
            Err(UsecaseError::Validation("Title cannot be empty.".into()))
        } else {
            Ok(trimmed.to_string())
        }
    }
}

