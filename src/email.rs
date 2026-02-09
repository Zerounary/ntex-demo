use std::env;

use lettre::message::{header::ContentType, Mailbox, Message};
use lettre::transport::smtp::authentication::Credentials;
use lettre::{AsyncSmtpTransport, AsyncTransport, Tokio1Executor};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum EmailError {
    #[error("missing env: {0}")]
    MissingEnv(&'static str),
    #[error("invalid env: {0}")]
    InvalidEnv(&'static str),
    #[error("invalid recipient email: {0}")]
    InvalidRecipient(String),
    #[error("build email failed")]
    BuildEmail,
    #[error("smtp transport error: {0}")]
    Transport(String),
}

fn read_env(key: &'static str) -> Result<String, EmailError> {
    env::var(key)
        .map(|v| v.trim().to_string())
        .map_err(|_| EmailError::MissingEnv(key))
        .and_then(|v| if v.is_empty() { Err(EmailError::MissingEnv(key)) } else { Ok(v) })
}

fn read_env_or(key: &'static str, default: String) -> String {
    env::var(key)
        .map(|v| v.trim().to_string())
        .ok()
        .filter(|v| !v.is_empty())
        .unwrap_or(default)
}

#[derive(Debug, Clone)]
pub struct EmailConfig {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub from_email: String,
    pub from_name: String,
}

impl EmailConfig {
    pub fn from_env() -> Result<Self, EmailError> {
        let host = read_env("SMTP_HOST")?;
        let port: u16 = read_env("SMTP_PORT")
            .map_err(|_| EmailError::MissingEnv("SMTP_PORT"))?
            .parse()
            .map_err(|_| EmailError::InvalidEnv("SMTP_PORT"))?;
        let username = read_env("SMTP_USERNAME")?;
        let password = read_env("SMTP_PASSWORD")?;
        let from_email = read_env_or("SMTP_FROM", username.clone());
        let from_name = read_env_or("SMTP_FROM_NAME", "NEBULA".to_string());

        Ok(Self {
            host,
            port,
            username,
            password,
            from_email,
            from_name,
        })
    }
}

#[derive(Clone)]
pub struct EmailClient {
    transport: AsyncSmtpTransport<Tokio1Executor>,
    from: Mailbox,
}

impl EmailClient {
    pub fn from_env() -> Result<Self, EmailError> {
        let cfg = EmailConfig::from_env()?;
        Self::from_config(&cfg)
    }

    pub fn from_config(cfg: &EmailConfig) -> Result<Self, EmailError> {
        let creds = Credentials::new(cfg.username.clone(), cfg.password.clone());
        let transport = AsyncSmtpTransport::<Tokio1Executor>::relay(&cfg.host)
            .map_err(|_| EmailError::InvalidEnv("SMTP_HOST"))?
            .port(cfg.port)
            .credentials(creds)
            .build();

        let from: Mailbox = format!("{} <{}>", cfg.from_name, cfg.from_email)
            .parse()
            .map_err(|_| EmailError::InvalidEnv("SMTP_FROM"))?;

        Ok(Self { transport, from })
    }

    pub async fn send_text(&self, to: &str, subject: &str, body: &str) -> Result<(), EmailError> {
        let to_mailbox: Mailbox = to
            .parse()
            .map_err(|_| EmailError::InvalidRecipient(to.to_string()))?;

        let email = Message::builder()
            .from(self.from.clone())
            .to(to_mailbox)
            .subject(subject)
            .header(ContentType::TEXT_PLAIN)
            .body(body.to_string())
            .map_err(|_| EmailError::BuildEmail)?;

        self.transport
            .send(email)
            .await
            .map(|_| ())
            .map_err(|e| EmailError::Transport(e.to_string()))
    }
}

pub async fn send_text_email(to: &str, subject: &str, body: &str) -> Result<(), EmailError> {
    let client = EmailClient::from_env()?;
    client.send_text(to, subject, body).await
}
