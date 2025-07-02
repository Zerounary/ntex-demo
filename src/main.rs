use ntex::web;
use ntex_files as fs;
use dotenv::dotenv;
use std::env;

#[web::post("/echo")]
async fn echo(req_body: String) -> impl web::Responder {
    web::HttpResponse::Ok().body(req_body)
}

async fn manual_hello() -> impl web::Responder {
    web::HttpResponse::Ok().body("Hey there!")
}

#[ntex::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok(); // Load .env file

    let port = env::var("PORT")
        .unwrap_or_else(|_| "8080".to_string())
        .parse()
        .expect("PORT must be a number");

    web::HttpServer::new(|| {

        web::App::new()
            .service(echo)
            .service(
                fs::Files::new("/", "./static")
                    .show_files_listing()
                    .index_file("index.html")
            )
            .route("/hey", web::get().to(manual_hello))
    })
    .bind(("127.0.0.1", port))?
    .run()
    .await
}