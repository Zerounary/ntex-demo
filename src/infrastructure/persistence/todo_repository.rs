use async_trait::async_trait;
use chrono::Utc;
use sea_orm::{
    ActiveModelTrait, DatabaseConnection, DbErr, EntityTrait, IntoActiveModel, Set,
};

use crate::application::errors::RepositoryError;
use crate::application::ports::TodoRepository;
use crate::domain::todo::{NewTodo, Todo, UpdateTodo};

use super::todo_entity;

pub struct TodoRepositoryImpl<'a> {
    db: &'a DatabaseConnection,
}

impl<'a> TodoRepositoryImpl<'a> {
    pub fn new(db: &'a DatabaseConnection) -> Self {
        Self { db }
    }

    fn map_err(err: DbErr) -> RepositoryError {
        RepositoryError::Persistence(err.to_string())
    }
}

#[async_trait]
impl<'a> TodoRepository for TodoRepositoryImpl<'a> {
    async fn list(&self) -> Result<Vec<Todo>, RepositoryError> {
        let models = todo_entity::Entity::find()
            .all(self.db)
            .await
            .map_err(Self::map_err)?;
        Ok(models.into_iter().map(Into::into).collect())
    }

    async fn find(&self, id: i32) -> Result<Option<Todo>, RepositoryError> {
        let model = todo_entity::Entity::find_by_id(id)
            .one(self.db)
            .await
            .map_err(Self::map_err)?;
        Ok(model.map(Into::into))
    }

    async fn create(&self, new_todo: NewTodo) -> Result<Todo, RepositoryError> {
        let active = todo_entity::ActiveModel {
            title: Set(new_todo.title),
            completed: Set(new_todo.completed),
            created_at: Set(Utc::now()),
            ..Default::default()
        };

        let model = active
            .insert(self.db)
            .await
            .map_err(Self::map_err)?;
        Ok(model.into())
    }

    async fn update(
        &self,
        id: i32,
        update: UpdateTodo,
    ) -> Result<Option<Todo>, RepositoryError> {
        let Some(model) = todo_entity::Entity::find_by_id(id)
            .one(self.db)
            .await
            .map_err(Self::map_err)?
        else {
            return Ok(None);
        };

        let mut active: todo_entity::ActiveModel = model.into_active_model();

        if let Some(title) = update.title {
            active.title = Set(title);
        }

        if let Some(completed) = update.completed {
            active.completed = Set(completed);
        }

        let updated = active
            .update(self.db)
            .await
            .map_err(Self::map_err)?;

        Ok(Some(updated.into()))
    }

    async fn delete(&self, id: i32) -> Result<bool, RepositoryError> {
        let result = todo_entity::Entity::delete_by_id(id)
            .exec(self.db)
            .await
            .map_err(Self::map_err)?;
        Ok(result.rows_affected > 0)
    }
}

