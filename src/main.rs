use actions::EClientTesting;
use actix_web::{web::{self, Data}, App, HttpServer};
use handlers::{application::{initialize_new_app_id, get_application_list, get_application, delete_application, update_application,}, index::get_app_list_of_indexes};
use handlers::document::{post_search, search, update_document, delete_document, get_document, create_bulk_documents};
use handlers::index::{get_index, create_index, update_mappings, get_mappings, delete_index};
mod middlewares;
use middlewares::cors::cors;

mod actions;
mod handlers;

/// Where should the main list be
/// The constant must be in lowercase, without space, and lowercase alphanumeric
pub const APPLICATION_LIST_NAME: &str = "user_apps";

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Debug mode
    std::env::set_var("RUST_LOG", "debug");
    env_logger::init();

    let client = Data::new(EClientTesting::new("http://127.0.0.1:9200"));

    // Start server
    HttpServer::new(move || {
        App::new()
        .wrap(cors())
        .service(
            web::scope("/api")
                .app_data(client.clone())
                .route("/app", web::post().to(initialize_new_app_id))
                .route("/apps", web::get().to(get_application_list))
                .route("/app/{app_id}", web::get().to(get_application))
                .route("/app", web::put().to(update_application))   
                .route("/app/{app_id}", web::delete().to(delete_application))
                
                .route("/index/{app_id}", web::post().to(create_index))
                .route("/index/{app_id}", web::get().to(get_index))
                .route("/index/list/{app_id}", web::get().to(get_app_list_of_indexes))
                .route("/index/mappings/{app_id}/{index}", web::get().to(get_mappings))
                .route("/index/mappings", web::put().to(update_mappings))
                .route("/index/{app_id}/{index}", web::delete().to(delete_index))
                
                .route("/document/{app_id}/{index}", web::post().to(create_bulk_documents))
                .route("/document/{app_id}/{index}/{document_id}", web::get().to(get_document))
                .route("/search/{app_id}/{index}", web::post().to(post_search))
                .route("/search/{app_id}/{index}", web::get().to(search))
                .route("/document/{app_id}/{index}/{document_id}", web::put().to(update_document))
                .route("/document/{app_id}/{index}/{document_id}", web::delete().to(delete_document))
        )
        })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await


}