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

        let num = request.num.unwrap_or(0);
        if num <= 0 {
            return Err(UsecaseError::Validation("num must be greater than 0".into()));
        }

        if request.cdk_type == crate::domain::cdk::CdkType::Bandwidth {
            let bw = request.bandwidth_mbps.unwrap_or(num);
            if bw <= 0 {
                return Err(UsecaseError::Validation(
                    "num/bandwidth_mbps must be greater than 0 for bandwidth cards".into(),
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

        let cdk = self
            .cdk_repo
            .get_cdk_by_code(&request.code)
            .await?
            .ok_or_else(|| UsecaseError::Validation("CDK not found".into()))?;

        // 兑换CDK
        let response = self.cdk_repo.redeem_cdk(request.clone()).await?;

        if cdk.cdk_type == crate::domain::cdk::CdkType::Bandwidth {
            let bw = cdk.bandwidth_mbps.unwrap_or(cdk.num);
            if bw <= 0 {
                return Err(UsecaseError::Validation("bandwidth must be positive".into()));
            }

            self.auth_repo
                .set_bandwidth_mbps(&request.user_id, bw)
                .await?;

            return Ok(CdkRedeemResponse {
                success: true,
                message: format!("CDK redeemed successfully. Bandwidth set to {} Mbps", bw),
                cdk_type: cdk.cdk_type,
                num: cdk.num,
                valid_until: None,
                remaining_minutes: None,
            });
        }

        if cdk.cdk_type == crate::domain::cdk::CdkType::Minute {
            let remaining = self.auth_repo.add_remaining_minutes(&request.user_id, cdk.num).await?;

            return Ok(CdkRedeemResponse {
                success: true,
                message: format!("CDK redeemed successfully. Remaining minutes: {}", remaining),
                cdk_type: cdk.cdk_type,
                num: cdk.num,
                valid_until: None,
                remaining_minutes: Some(remaining),
            });
        }

        // Day/Month/Year: 更新用户有效期（num 表示多少天/月/年）
        if cdk.num <= 0 {
            return Err(UsecaseError::Validation("num must be positive for pass cards".into()));
        }
        let minutes = cdk.cdk_type.duration_minutes() * cdk.num;
        if minutes <= 0 {
            return Err(UsecaseError::Validation("invalid pass duration computed".into()));
        }

        let new_valid_until = if user.valid_until > chrono::Utc::now() {
            user.valid_until + chrono::Duration::minutes(minutes)
        } else {
            chrono::Utc::now() + chrono::Duration::minutes(minutes)
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
            cdk_type: cdk.cdk_type,
            num: cdk.num,
            valid_until: Some(new_valid_until),
            remaining_minutes: None,
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
        let remaining_minutes = self.auth_repo.get_remaining_minutes(&request.user_id).await?;
        let is_valid = is_paid || remaining_minutes > 0;

        let billing_mode = if is_paid {
            "pass".to_string()
        } else if remaining_minutes > 0 {
            "minute".to_string()
        } else {
            "none".to_string()
        };

        let message = if is_paid {
            format!(
                "Account is valid until {}",
                user.valid_until.format("%Y-%m-%d %H:%M:%S")
            )
        } else if remaining_minutes > 0 {
            format!("Account has remaining minutes: {}", remaining_minutes)
        } else {
            "Account has expired or is not paid".to_string()
        };

        Ok(AccountValidationResponse {
            is_valid,
            is_paid,
            valid_until: Some(user.valid_until),
            remaining_minutes,
            billing_mode,
            message,
        })
    }
}




