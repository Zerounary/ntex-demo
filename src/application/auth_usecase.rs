use uuid::Uuid;

use crate::domain::auth::{AccountLoginRequest, AccountLoginResponse, TicketStatus, WechatTicket};

use super::errors::UsecaseError;
use super::ports::AuthRepository;

pub struct AuthUseCase<A> {
    auth_repo: A,
}

impl<A> AuthUseCase<A> {
    pub fn new(auth_repo: A) -> Self {
        Self { auth_repo }
    }
}

impl<A> AuthUseCase<A>
where
    A: AuthRepository,
{
    pub async fn create_ticket(&self, scene: String) -> Result<WechatTicket, UsecaseError> {
        let ticket_id = Uuid::new_v4().to_string();
        let expires_in = 120;
        let ticket = WechatTicket {
            ticket_id: ticket_id.clone(),
            qr_code_url: format!("https://example.com/qrcode/{ticket_id}"),
            expires_in,
            status: TicketStatus::Pending,
            scene,
            success: false,
            user: None,
        };
        self.auth_repo.insert_ticket(&ticket).await?;
        Ok(ticket)
    }

    pub async fn ticket_status(&self, ticket_id: &str) -> Result<WechatTicket, UsecaseError> {
        let mut ticket = self
            .auth_repo
            .get_ticket(ticket_id)
            .await?
            .ok_or(UsecaseError::NotFound("Ticket"))?;

        if ticket.expires_in <= 0 {
            ticket.status = TicketStatus::Expired;
            ticket.success = false;
            self.auth_repo.save_ticket(&ticket).await?;
            return Err(UsecaseError::Expired);
        }

        Ok(ticket)
    }

    pub async fn account_login(
        &self,
        request: AccountLoginRequest,
    ) -> Result<AccountLoginResponse, UsecaseError> {
        match self.auth_repo.login_account(request).await {
            Ok(response) => Ok(response),
            Err(crate::application::errors::RepositoryError::Persistence(msg))
                if msg == "invalid credentials" =>
            {
                Err(UsecaseError::Unauthorized)
            }
            Err(err) => Err(UsecaseError::Repository(err)),
        }
    }
}
