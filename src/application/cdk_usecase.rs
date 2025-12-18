use crate::domain::cdk::{
    AccountValidationRequest, AccountValidationResponse, CdkCode, CdkGenerateRequest,
    CdkRedeemRequest, CdkRedeemResponse,
};

use super::errors::UsecaseError;
use super::ports::{AuthRepository, CdkRepository};

pub struct CdkUseCase<C, A> {
    cdk_repo: C,
    auth_repo: A,
}

impl<C, A> CdkUseCase<C, A> {
    pub fn new(cdk_repo: C, auth_repo: A) -> Self {
        Self { cdk_repo, auth_repo }
    }
}

impl<C, A> CdkUseCase<C, A>
where
    C: CdkRepository,
    A: AuthRepository,
{
    pub async fn generate_cdks(
        &self,
        request: CdkGenerateRequest,
    ) -> Result<Vec<CdkCode>, UsecaseError> {
        if request.count == 0 {
            return Err(UsecaseError::Validation("count must be greater than 0".into()));
        }

        if request.count > 1000 {
            return Err(UsecaseError::Validation(
                "count cannot exceed 1000".into(),
            ));
        }

        if request.cdk_type == crate::domain::cdk::CdkType::Minute {
            if request.duration_minutes.is_none() || request.duration_minutes.unwrap() <= 0 {
                return Err(UsecaseError::Validation(
                    "duration_minutes is required for minute cards".into(),
                ));
            }
        }

        let cdks = self.cdk_repo.generate_cdks(request).await?;
        Ok(cdks)
    }

    pub async fn redeem_cdk(
        &self,
        request: CdkRedeemRequest,
    ) -> Result<CdkRedeemResponse, UsecaseError> {
        // 验证用户是否存在
        let user = self
            .auth_repo
            .get_user_by_id(&request.user_id)
            .await?
            .ok_or_else(|| UsecaseError::Validation("user not found".into()))?;

        // 兑换CDK
        let response = self.cdk_repo.redeem_cdk(request.clone()).await?;

        // 更新用户有效期
        let new_valid_until = if user.valid_until > chrono::Utc::now() {
            // 如果用户还有有效期，累加时间
            user.valid_until + chrono::Duration::minutes(response.duration_minutes)
        } else {
            // 如果用户已过期，从当前时间开始计算
            response.valid_until
        };

        self.auth_repo
            .update_user_valid_until(&request.user_id, new_valid_until)
            .await?;

        Ok(CdkRedeemResponse {
            success: true,
            message: format!(
                "CDK redeemed successfully. Valid until: {}",
                new_valid_until.format("%Y-%m-%d %H:%M:%S")
            ),
            duration_minutes: response.duration_minutes,
            valid_until: new_valid_until,
        })
    }

    pub async fn list_cdks(
        &self,
        status: Option<&str>,
    ) -> Result<Vec<CdkCode>, UsecaseError> {
        let cdks = self.cdk_repo.list_cdks(status).await?;
        Ok(cdks)
    }

    pub async fn validate_account(
        &self,
        request: AccountValidationRequest,
    ) -> Result<AccountValidationResponse, UsecaseError> {
        let user = self
            .auth_repo
            .get_user_by_id(&request.user_id)
            .await?
            .ok_or_else(|| UsecaseError::Validation("user not found".into()))?;

        let now = chrono::Utc::now();
        let is_paid = user.valid_until > now;
        let is_valid = is_paid;

        let message = if is_paid {
            format!(
                "Account is valid until {}",
                user.valid_until.format("%Y-%m-%d %H:%M:%S")
            )
        } else {
            "Account has expired or is not paid".to_string()
        };

        Ok(AccountValidationResponse {
            is_valid,
            is_paid,
            valid_until: Some(user.valid_until),
            message,
        })
    }
}




