use std::env;

use dotenv::dotenv;
use ntex_demo::email;

#[tokio::main]
async fn main() -> Result<(), email::EmailError> {
    dotenv().ok();

    let mut args = env::args().skip(1);
    let to = args
        .next()
        .or_else(|| env::var("TEST_EMAIL_TO").ok())
        .ok_or(email::EmailError::MissingEnv("TEST_EMAIL_TO"))?;

    let subject = args
        .next()
        .unwrap_or_else(|| "SMTP 配置测试".to_string());

    let body = if let Some(body) = args.next() {
        body
    } else {
        "这是来自 send_email_tool 示例的测试邮件。".to_string()
    };

    email::send_text_email(&to, &subject, &body).await?;
    println!("测试邮件已发送至 {to}");

    Ok(())
}
