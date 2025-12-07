use crate::domain::accelerator::{BootstrapPayload, Profile};

use super::errors::UsecaseError;
use super::ports::AcceleratorRepository;

pub struct AcceleratorUseCase<A> {
    accelerator_repo: A,
}

impl<A> AcceleratorUseCase<A> {
    pub fn new(accelerator_repo: A) -> Self {
        Self { accelerator_repo }
    }
}

impl<A> AcceleratorUseCase<A>
where
    A: AcceleratorRepository,
{
    pub async fn bootstrap(&self) -> Result<BootstrapPayload, UsecaseError> {
        let payload = self.accelerator_repo.bootstrap().await?;
        Ok(payload)
    }

    pub async fn sync_profiles(&self, profiles: Vec<Profile>) -> Result<(), UsecaseError> {
        if profiles.is_empty() {
            return Err(UsecaseError::Validation(
                "profiles payload cannot be empty".into(),
            ));
        }
        self.accelerator_repo.upsert_profiles(profiles).await?;
        Ok(())
    }
}
