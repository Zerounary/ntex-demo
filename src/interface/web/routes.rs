use ntex::web;

use super::handlers;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api")
            .service(
                web::resource("/todos")
                    .route(web::get().to(handlers::list_todos))
                    .route(web::post().to(handlers::create_todo)),
            )
            .service(
                web::resource("/todos/{id}")
                    .route(web::get().to(handlers::get_todo))
                    .route(web::put().to(handlers::update_todo))
                    .route(web::delete().to(handlers::delete_todo)),
            ),
    );
}

