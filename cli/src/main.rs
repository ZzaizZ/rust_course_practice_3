use clap::{Parser, Subcommand};

#[derive(Subcommand, Debug)]
enum Command {
    Register {
        username: String,
        password: String,
        email: String,
    },
    Login {
        username: String,
        password: String,
    },

    CreatePost {
        title: String,
        content: String,
    },
    GetPost {
        uuid: String,
    },
    UpdatePost {
        uuid: String,
        title: String,
        content: String,
    },
    DeletePost {
        uuid: String,
    },
    ListPosts {
        page_size: u8,
        page: u32,
    },
}

#[derive(Parser, Debug)]
struct Args {
    #[arg(short, long)]
    use_grpc: bool,

    #[arg(short, long, default_value = "http://localhost:8080")]
    server: String,

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
        Command::Register {
            username,
            password,
            email,
        } => {
            client.register(&username, &email, &password).await?;
            println!("User registered: {}", username);
        }
        Command::Login { username, password } => {
            client.login(&username, &password).await?;
            println!("User logged in: {}", username);
            if let Ok(Some(token)) = client.get_token().await {
                std::fs::write(".blog_token", token).expect("Failed to save cookie to file");
                println!("Cookie saved to .blog_token file");
            }
        }
        Command::CreatePost { title, content } => {
            let token =
                std::fs::read_to_string(".blog_token").expect("Failed to read token from file");
            client.setup_token(&token).await.expect("Unauthorized");

            client.create_post(&title, &content).await?;
            println!("Post created: {}", title);
        }
        Command::GetPost { uuid } => {
            let post = client.get_post(&uuid).await?;
            println!("Post retrieved: {}", post.title);
            println!("{}", post.content);
        }
        Command::UpdatePost {
            uuid,
            title,
            content,
        } => {
            let token =
                std::fs::read_to_string(".blog_token").expect("Failed to read token from file");
            client.setup_token(&token).await?;

            client.update_post(&uuid, &title, &content).await?;
            println!("Post updated: {}", uuid);
        }
        Command::DeletePost { uuid } => {
            let token =
                std::fs::read_to_string(".blog_token").expect("Failed to read token from file");
            client.setup_token(&token).await?;

            client.delete_post(&uuid).await?;
            println!("Post deleted: {}", uuid);
        }
        Command::ListPosts { page_size, page } => {
            let token =
                std::fs::read_to_string(".blog_token").expect("Failed to read token from file");
            client.setup_token(&token).await?;
            let posts = client.list_posts(page_size, page).await?;
            println!("Posts (page {}, size {}):", page, page_size);
            for post in posts {
                println!("  - {}: {}", post.id, post.title);
            }
        }
    }

    Ok(())
}
