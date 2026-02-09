#[derive(Debug, Clone)]
pub struct RegisterDto {
    pub username: String,
    pub password: String,
    pub email: String,
}

#[derive(Debug, Clone)]
pub struct LoginDto {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Clone)]
pub struct TokenDto {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_in: i64,
}

#[derive(Debug, Clone)]
pub struct UserInfoDto {
    pub user_id: String,
    pub username: String,
    pub email: String,
}
