//! # CLI
//!
//! Утилита командной строки для взаимодействия с блог-платформой.
//!
//! ## Возможности
//!
//! - Регистрация и вход пользователей
//! - Управление постами (создание, чтение, обновление, удаление)
//! - Поддержка HTTP и gRPC транспорта
//! - Сохранение токенов в файл `.blog_token`
//! - Безопасный ввод пароля с консоли (без отображения символов)
//!
//! ## Примеры использования
//!
//! ```bash
//! # Регистрация пользователя
//! cargo run --bin cli -- register -u user -e user@example.com -p password123
//!
//! # Вход с паролем в аргументах (сохраняет токен в .blog_token)
//! cargo run --bin cli -- login -u user -p password123
//!
//! # Вход с запросом пароля с консоли (пароль не будет виден при вводе)
//! cargo run --bin cli -- login -u user
//!
//! # Создание поста
//! cargo run --bin cli -- create-post -t "Title" -c "Content"
//!
//! # Список постов
//! cargo run --bin cli -- list-posts --page-size 10 --page 0
//!
//! # Использование gRPC вместо HTTP
//! cargo run --bin cli -- --use-grpc --server http://localhost:50051 list-posts --page-size 10 --page 0
//! ```

use clap::{Parser, Subcommand};

/// Доступные команды CLI.
#[derive(Subcommand, Debug)]
enum Command {
    /// Регистрация нового пользователя
    Register(RegisterArgs),
    /// Вход пользователя в систему
    Login(LoginArgs),
    /// Создание нового поста
    CreatePost(CreatePostArgs),
    /// Получение поста по ID
    GetPost(GetPostArgs),
    /// Обновление существующего поста
    UpdatePost(UpdatePostArgs),
    /// Удаление поста
    DeletePost(DeletePostArgs),
    /// Получение списка постов с пагинацией
    ListPosts(ListPostsArgs),
}

#[derive(Parser, Debug)]
struct RegisterArgs {
    #[arg(short, long, required = true)]
    username: String,
    #[arg(short, long, required = true)]
    password: String,
    #[arg(short, long, required = true)]
    email: String,
}

#[derive(Parser, Debug)]
struct LoginArgs {
    #[arg(short, long, required = true)]
    username: String,
    #[arg(short, long)]
    password: Option<String>,
}

#[derive(Parser, Debug)]
struct CreatePostArgs {
    #[arg(short, long, required = true)]
    title: String,
    #[arg(short, long, required = true)]
    content: String,
}

#[derive(Parser, Debug)]
struct GetPostArgs {
    #[arg(short, long, required = true)]
    uuid: String,
}

#[derive(Parser, Debug)]
struct UpdatePostArgs {
    #[arg(short, long, required = true)]
    uuid: String,
    #[arg(short, long, required = true)]
    title: String,
    #[arg(short, long, required = true)]
    content: String,
}

#[derive(Parser, Debug)]
struct DeletePostArgs {
    #[arg(short, long, required = true)]
    uuid: String,
}

#[derive(Parser, Debug)]
struct ListPostsArgs {
    #[arg(long, default_value = "10")]
    page_size: u32,
    #[arg(long, default_value = "0")]
    page: u32,
}

/// Загружает данные аутентификации из файла.
fn load_auth_data() -> Result<client::types::AuthData, Box<dyn std::error::Error>> {
    let json = std::fs::read_to_string(".blog_token")?;
    let auth_data: client::types::AuthData = serde_json::from_str(&json)?;
    Ok(auth_data)
}

/// Сохраняет данные аутентификации в файл.
fn save_auth_data(auth_data: &client::types::AuthData) -> Result<(), Box<dyn std::error::Error>> {
    let json = serde_json::to_string(auth_data)?;
    std::fs::write(".blog_token", json)?;
    Ok(())
}

/// Аргументы командной строки.
#[derive(Parser, Debug)]
#[command(author, version, about = "CLI для блог-платформы", long_about = None)]
struct Args {
    /// Использовать gRPC вместо HTTP
    #[arg(short, long)]
    use_grpc: bool,

    /// URL сервера
    #[arg(short, long, default_value = "http://localhost:8080")]
    server: String,

    /// Команда для выполнения
    #[command(subcommand)]
    command: Command,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let client: Box<dyn client::blog_client::BlogClient> = if args.use_grpc {
        Box::new(
            client::grpc_client::GrpcClient::new(args.server.clone())
                .await
                .expect("Failed to create gRPC client"),
        )
    } else {
        Box::new(
            client::http_client::HttpClient::new(args.server.clone())
                .await
                .expect("Failed to create HTTP client"),
        )
    };

    println!(
        "Client initialized using {} transport",
        if args.use_grpc { "gRPC" } else { "HTTP" }
    );

    match args.command {
        Command::Register(args) => {
            client
                .register(&args.username, &args.email, &args.password)
                .await?;
            println!("User registered: {}", args.username);
        }
        Command::Login(args) => {
            let password = if let Some(pwd) = args.password {
                pwd
            } else {
                rpassword::prompt_password("Password: ")?
            };

            client.login(&args.username, &password).await?;
            println!("User logged in: {}", args.username);
            if let Ok(Some(auth_data)) = client.get_auth_data().await {
                save_auth_data(&auth_data)?;
                println!("Tokens saved to .blog_token file");
            }
        }
        Command::CreatePost(args) => {
            let auth_data = load_auth_data()?;
            client.setup_auth_data(&auth_data).await?;

            client.create_post(&args.title, &args.content).await?;
            println!("Post created: {}", args.title);
        }
        Command::GetPost(args) => {
            let post = client.get_post(&args.uuid).await?;
            println!("Post retrieved: {}", post.title);
            println!("{}", post.content);
        }
        Command::UpdatePost(args) => {
            let auth_data = load_auth_data()?;
            client.setup_auth_data(&auth_data).await?;

            client
                .update_post(&args.uuid, &args.title, &args.content)
                .await?;
            println!("Post updated: {}", args.uuid);
        }
        Command::DeletePost(args) => {
            let auth_data = load_auth_data()?;
            client.setup_auth_data(&auth_data).await?;

            client.delete_post(&args.uuid).await?;
            println!("Post deleted: {}", args.uuid);
        }
        Command::ListPosts(args) => {
            let auth_data = load_auth_data()?;
            client.setup_auth_data(&auth_data).await?;

            let posts = client.list_posts(args.page_size, args.page).await?;
            println!("Posts (page {}, size {}):", args.page, args.page_size);
            for post in posts {
                println!("  - {}: {}", post.id, post.title);
            }
        }
    }

    Ok(())
}
