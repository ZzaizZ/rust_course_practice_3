use std::sync::Arc;

use actix_cors::Cors;
use actix_web::{App, HttpServer, web};
use actix_web_httpauth::middleware::HttpAuthentication;
use api::blog_server::BlogServer;
use server::{
    application::{auth::AuthApplication, post::PostApplication},
    data::pgrepo::PgUserRepository,
    domain::services::auth::AuthService,
    infrastructure::config::Config,
    presentation::grpc::BlogServiceImpl,
    presentation::http::handlers::{
        AppState, create_post, delete_post, get_post, list_posts, login, refresh_token, register,
        update_post,
    },
    presentation::http::middleware::jwt_validator,
};
use tonic::transport::Server;
use tracing::{error, info};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cfg = Config::from_file("config.yml").expect("Failed to load configuration");

    // Инициализация tracing
    tracing_subscriber::fmt()
        .with_max_level(cfg.log_level.parse().unwrap_or(tracing::Level::INFO))
        .with_target(true)
        .with_thread_ids(false)
        .with_line_number(true)
        .init();

    info!("Starting server initialization");
    info!("Configuration loaded successfully");

    let repo = PgUserRepository::new(&cfg.db_connection_string)
        .await
        .map_err(|e| {
            error!("Failed to create repository: {}", e);
            e
        })
        .expect("Failed to create repository");
    let repo = Arc::new(repo);
    info!("Database repository initialized");

    let auth_service = AuthService::new(
        chrono::Duration::seconds(cfg.jwt_expiration_seconds),
        cfg.jwt_secret.as_bytes(),
    );
    let auth_service = Arc::new(auth_service);
    info!("Auth service initialized");

    let auth_app = Arc::new(AuthApplication::new(repo.clone(), auth_service.clone()));
    let post_app = Arc::new(PostApplication::new(repo.clone()));

    let app_state = web::Data::new(AppState {
        auth_app: auth_app.clone(),
        post_app: post_app.clone(),
    });
    let auth_service_data = web::Data::from(auth_service.clone());

    let http_addr = format!("127.0.0.1:{}", cfg.server_port);
    let grpc_addr = format!("127.0.0.1:{}", cfg.grpc_port)
        .parse()
        .expect("Invalid gRPC address");

    info!("Starting HTTP server at http://{}", http_addr);
    info!("Starting gRPC server at {}", grpc_addr);

    let cors_origin = cfg.cors_origin.clone();

    // Запускаем gRPC сервер в отдельной задаче
    let grpc_service = BlogServiceImpl::new(auth_app, post_app, auth_service);
    let grpc_server = tokio::spawn(async move {
        Server::builder()
            .add_service(BlogServer::new(grpc_service))
            .serve(grpc_addr)
            .await
            .map_err(|e| {
                error!("gRPC server error: {}", e);
                e
            })
    });

    // Запускаем HTTP сервер
    let http_server = HttpServer::new(move || {
        let cors = Cors::default()
            .allowed_origin(&cors_origin)
            .allowed_methods(vec!["GET", "POST", "PUT", "DELETE", "OPTIONS"])
            .allow_any_header()
            .max_age(3600);

        // Создаём middleware для JWT аутентификации
        let auth_middleware = HttpAuthentication::bearer(jwt_validator);

        App::new()
            .app_data(app_state.clone())
            .app_data(auth_service_data.clone())
            .wrap(tracing_actix_web::TracingLogger::default())
            .wrap(cors)
            // Публичные маршруты (без аутентификации)
            .service(register)
            .service(login)
            .service(refresh_token)
            .service(list_posts)
            .service(get_post)
            // Защищённые маршруты (требуют JWT токен)
            .service(
                web::scope("")
                    .wrap(auth_middleware)
                    .service(create_post)
                    .service(update_post)
                    .service(delete_post),
            )
    })
    .bind(&http_addr)
    .map_err(|e| {
        error!("Failed to bind to {}: {}", http_addr, e);
        e
    })?
    .run();

    // Ждем завершения обоих серверов
    tokio::select! {
        res = http_server => {
            res?;
        }
        res = grpc_server => {
            res??;
        }
    }

    Ok(())
}
