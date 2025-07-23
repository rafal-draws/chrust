extern crate dotenv;

mod db;
mod http;
mod ml;



use back::config::{self, create_server_data_dirs, create_upload_dir, start_4hourly_task};

use axum::extract::DefaultBodyLimit;
use db::db_conn::{self};
use dotenv::dotenv;

use tower_http::services::ServeDir;

use axum::routing::{get, post};
use axum::Router;

use tracing_subscriber::fmt;

use crate::http::handlers::delete::delete_upload;
use crate::http::handlers::help::help;
use crate::http::handlers::profile::get_user_data;
use crate::http::handlers::register::{register_user, terms_of_use, user_form, user_registered};
use crate::http::handlers::track_menu::track_menu;
use crate::http::handlers::upload::upload_track;

#[allow(unused)]
#[tokio::main]
async fn main() -> Result<(), Box<std::io::Error>> {

    dotenv().ok();
    let subscriber = fmt().with_line_number(true).with_file(true).finish();

    println!("DATABASE IS: {:?}", std::env::var("DATABASE_URL").expect("should exist"));
    tracing::subscriber::set_global_default(subscriber).expect("setting tracing default failed");
    let pool = db_conn::get_pool().await;

    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Migration should be possible");

    // clear_server_data().unwrap();
    tracing::info!("server_data cleared!");
    // drop_all_users().await.unwrap();
    tracing::info!("uploads in db cleared!");
    create_server_data_dirs("server_data")?;
    tracing::info!("Directories created");


    create_upload_dir().await;

    // removal of records from db - according job present on backend-etl
    start_4hourly_task().await;


    let app = app()
        .layer(config::create_cors_layers())
        .layer(DefaultBodyLimit::max(250 * 1024 * 1024));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();

    println!("listening on http://{:?}/", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();

    Ok(())
}

fn app() -> Router {
    tracing::info!("SETUP - CREATING ENDPOINTS");
    Router::new()
        .route("/", get(user_form))
        .route("/terms", get(terms_of_use))
        .route("/register", post(register_user).get(user_registered)) //todo check if register_user is needed
        .route("/profile", get(get_user_data))
        .route("/upload", post(upload_track))
        .route("/delete/{upload_uuid}", post(delete_upload))
        .route("/track/{upload_name}", get(track_menu))
        .route("/help/{keyword}", get(help))
        
        .nest_service("/server_data", ServeDir::new(
            std::env::var("SERVER_DATA").unwrap()))
        .nest_service("/static", ServeDir::new("static"))
}

