use client::{blog_client::BlogClient, grpc_client::GrpcClient};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Создаем gRPC клиента
    let client = GrpcClient::new("http://localhost:50051".to_string()).await?;

    println!("gRPC клиент создан");

    // Регистрация пользователя
    println!("\nРегистрация пользователя...");
    client
        .register("test_user_grpc", "test_grpc@example.com", "password123")
        .await?;
    println!("Пользователь зарегистрирован");

    // Вход
    println!("\nВход в систему...");
    let user_id = client.login("test_user_grpc", "password123").await?;
    println!("Вход выполнен! User ID: {}", user_id);

    // Получаем текущий токен (если нужно)
    if let Some(token) = client.get_token().await {
        println!("Access token: {}...", &token[..20.min(token.len())]);
    }

    // Создание поста
    println!("\nСоздание поста...");
    let post_id = client
        .create_post(
            "My First gRPC Post",
            "This is the content of my first gRPC post!",
        )
        .await?;
    println!("Пост создан с ID: {}", post_id);

    // Получение поста
    println!("\nПолучение поста...");
    let post = client.get_post(&post_id.to_string()).await?;
    println!("Пост получен:");
    println!("  Title: {}", post.title);
    println!("  Content: {}", post.content);
    println!("  Created at: {}", post.created_at);

    // Обновление поста
    println!("\nОбновление поста...");
    client
        .update_post(
            &post_id.to_string(),
            "Updated gRPC Title",
            "Updated content via gRPC!",
        )
        .await?;
    println!("Пост обновлен");

    // Получение обновленного поста
    println!("\nПолучение обновленного поста...");
    let updated_post = client.get_post(&post_id.to_string()).await?;
    println!("Обновленный пост:");
    println!("  Title: {}", updated_post.title);
    println!("  Content: {}", updated_post.content);

    // Список постов
    println!("\nПолучение списка постов...");
    let posts = client.list_posts(10, 0).await?;
    println!("Найдено постов: {}", posts.len());
    for (i, post) in posts.iter().enumerate() {
        println!("  {}. {} - {}", i + 1, post.title, post.id);
    }

    // Удаление поста
    println!("\nУдаление поста...");
    client.delete_post(&post_id.to_string()).await?;
    println!("Пост удален");

    println!("\n✅ Все операции выполнены успешно!");
    println!("\nℹ️  Примечание: Токен автоматически проверялся и обновлялся при необходимости");

    Ok(())
}
