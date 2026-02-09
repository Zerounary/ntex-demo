use lettre::message::header::ContentType;
use lettre::transport::smtp::authentication::Credentials;
use lettre::{Message, SmtpTransport, Transport};

pub fn main() {
    let email = Message::builder()
        .from("NoBody <zerounary@163.com>".parse().unwrap())
        .reply_to("Yuin <850657663@qq.com>".parse().unwrap())
        .to("Hei <850657663@qq.com>".parse().unwrap())
        .subject("Happy new year")
        .header(ContentType::TEXT_PLAIN)
        .body(String::from("Be happy!"))
        .unwrap();

    let creds = Credentials::new("zerounary@163.com".to_owned(), "ERPcS7hqVAhevAQR".to_owned());

    // Open a remote connection to gmail
    let mailer = SmtpTransport::relay("smtp.163.com")
        .unwrap()
        .credentials(creds)
        .build();

    // Send the email
    match mailer.send(&email) {
        Ok(_) => println!("Email sent successfully!"),
        Err(e) => panic!("Could not send email: {e:?}"),
    }
}