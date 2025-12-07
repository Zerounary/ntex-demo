use crate::domain::content::{DashboardPayload, LibraryPayload, NavigationConfig, SettingsMeta};

use super::errors::UsecaseError;
use super::ports::ConfigRepository;

const DASHBOARD_KEY: &str = "dashboard";
const LIBRARY_KEY: &str = "library";
const SETTINGS_KEY: &str = "settings_meta";
const NAVIGATION_KEY: &str = "navigation";

pub struct ContentUseCase<C> {
    config_repo: C,
}

impl<C> ContentUseCase<C> {
    pub fn new(config_repo: C) -> Self {
        Self { config_repo }
    }
}

impl<C> ContentUseCase<C>
where
    C: ConfigRepository,
{
    pub async fn dashboard(&self) -> Result<DashboardPayload, UsecaseError> {
        self.fetch_and_deserialize(DASHBOARD_KEY).await
    }

    pub async fn library(&self) -> Result<LibraryPayload, UsecaseError> {
        self.fetch_and_deserialize(LIBRARY_KEY).await
    }

    pub async fn settings(&self) -> Result<SettingsMeta, UsecaseError> {
        self.fetch_and_deserialize(SETTINGS_KEY).await
    }

    pub async fn navigation(&self) -> Result<NavigationConfig, UsecaseError> {
        self.fetch_and_deserialize(NAVIGATION_KEY).await
    }

    async fn fetch_and_deserialize<T>(&self, key: &'static str) -> Result<T, UsecaseError>
    where
        T: for<'de> serde::Deserialize<'de>,
    {
        let value = self
            .config_repo
            .get_entry(key)
            .await?
            .ok_or(UsecaseError::NotFound(key))?;
        serde_json::from_value::<T>(value).map_err(|err| {
            UsecaseError::Repository(super::errors::RepositoryError::Serialization(
                err.to_string(),
            ))
        })
    }
}
