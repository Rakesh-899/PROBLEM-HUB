mod handlers;
mod models;
mod middleware;

use actix_web::{web, App, HttpServer, middleware::Logger};
use actix_cors::Cors;
use env_logger;
use dotenv::dotenv;
use sqlx::{postgres::{PgPool, PgPoolOptions}};
use handlers::{signup, login, protected_route, reset_password, get_profile, send_otp, verify_otp};
use middleware::AuthMiddleware;


//Create db functions for connection
pub async fn establish_db_connection() -> Result<PgPool, sqlx::Error> {
    let db_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    PgPoolOptions::new()
        .max_connections(5)
        .connect(&db_url)
        .await
}


#[actix_web::main]
async fn main() -> Result<(), std::io::Error> {
    // server
    if std::env::var("RUST_LOG").is_err() {
        unsafe {
            std::env::set_var("RUST_LOG", "info");
        }
    }
    dotenv().ok();
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    //Database connection 
    let pool = establish_db_connection()
        .await
        .expect("Failed to connect to database");

    HttpServer::new(move|| {
        App::new()
        .app_data(web::Data::new(pool.clone()))
        .wrap(Logger::default())
        .wrap(Cors::permissive())
        .service(signup)
        .service(login)
        .service(reset_password)
        .service(send_otp)
        .service(verify_otp)
        .service(
            web::scope("/api") 
                .wrap(AuthMiddleware)
                .service(protected_route)
                .service(get_profile)
        )

    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
