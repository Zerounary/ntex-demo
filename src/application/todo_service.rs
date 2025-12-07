use chrono::Utc;
use sea_orm::{ActiveModelTrait, DatabaseConnection, DbErr, EntityTrait, Set};

use crate::domain::todo;

pub struct TodoService<'a> {
    db: &'a DatabaseConnection,
}

impl<'a> TodoService<'a> {
    pub fn new(db: &'a DatabaseConnection) -> Self {
        Self { db }
    }

    pub async fn list(&self) -> Result<Vec<todo::Model>, DbErr> {
        todo::Entity::find().all(self.db).await
    }

    pub async fn find(&self, id: i32) -> Result<Option<todo::Model>, DbErr> {
        todo::Entity::find_by_id(id).one(self.db).await
    }

    pub async fn create(&self, title: String, completed: bool) -> Result<todo::Model, DbErr> {
        let active_model = todo::ActiveModel {
            title: Set(title),
            completed: Set(completed),
            created_at: Set(Utc::now()),
            ..Default::default()
        };

        active_model.insert(self.db).await
    }

    pub async fn update(
        &self,
        id: i32,
        title: Option<String>,
        completed: Option<bool>,
    ) -> Result<Option<todo::Model>, DbErr> {
        let existing = match self.find(id).await? {
            Some(model) => model,
            None => return Ok(None),
        };

        let mut model: todo::ActiveModel = existing.into();

        if let Some(title) = title {
            model.title = Set(title);
        }

        if let Some(completed) = completed {
            model.completed = Set(completed);
        }

        let todo = model.update(self.db).await?;
        Ok(Some(todo))
    }

    pub async fn delete(&self, id: i32) -> Result<bool, DbErr> {
        let result = todo::Entity::delete_by_id(id).exec(self.db).await?;
        Ok(result.rows_affected > 0)
    }

    pub fn validate_title(input: &str) -> Result<String, String> {
        let trimmed = input.trim();
        if trimmed.is_empty() {
            Err("Title cannot be empty.".into())
        } else {
            Ok(trimmed.to_string())
        }
    }
}

