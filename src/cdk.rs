use crate::application::ports::AuthRepository;
use crate::domain::cdk::CdkType;

pub async fn grant_cdk_reward<A>(
    auth_repo: &A,
    user_id: i64,
    raw_cdk_type: &str,
    num: i64,
) -> Result<(), String>
where
    A: AuthRepository,
{
    if num <= 0 {
        return Err("reward num must be positive".to_string());
    }

    let cdk_type = raw_cdk_type.trim().to_lowercase();
    if cdk_type.is_empty() {
        return Err("cdkType is required".to_string());
    }

    if cdk_type == "bandwidth" {
        auth_repo
            .set_bandwidth_mbps(user_id, num)
            .await
            .map_err(|e| format!("set_bandwidth_mbps failed: {:?}", e))?;
        return Ok(());
    }

    if cdk_type == "minute" {
        auth_repo
            .add_remaining_minutes(user_id, num)
            .await
            .map_err(|e| format!("add_remaining_minutes failed: {:?}", e))?;
        return Ok(());
    }

    let cdk_type_enum = CdkType::from_str(&cdk_type)
        .ok_or_else(|| format!("invalid cdkType: {}", cdk_type))?;
    if matches!(cdk_type_enum, CdkType::Minute | CdkType::Bandwidth) {
        return Err(format!("invalid pass-type cdkType: {}", cdk_type));
    }

    let user = auth_repo
        .get_user_by_id(user_id)
        .await
        .map_err(|e| format!("get_user_by_id failed: {:?}", e))?
        .ok_or_else(|| "user not found".to_string())?;

    let duration_minutes = cdk_type_enum.duration_minutes() * num;
    if duration_minutes <= 0 {
        return Err("invalid duration computed for pass reward".to_string());
    }

    let now = chrono::Utc::now();
    let next = if user.valid_until > now {
        user.valid_until + chrono::Duration::minutes(duration_minutes)
    } else {
        now + chrono::Duration::minutes(duration_minutes)
    };

    auth_repo
        .update_user_valid_until(user_id, next)
        .await
        .map_err(|e| format!("update_user_valid_until failed: {:?}", e))?;

    Ok(())
}