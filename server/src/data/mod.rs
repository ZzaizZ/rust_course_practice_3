pub mod pgrepo;

use crate::domain::entities::errors::DomainError;

impl From<sqlx::Error> for DomainError {
    fn from(error: sqlx::Error) -> Self {
        match error {
            sqlx::Error::RowNotFound => {
                DomainError::RepositoryError("Record not found".to_string())
            }
            sqlx::Error::Database(db_err) => {
                // Check for specific database errors
                if let Some(code) = db_err.code() {
                    // PostgreSQL unique violation error code
                    if code == "23505" {
                        return DomainError::RepositoryError(
                            "Duplicate entry: constraint violation".to_string(),
                        );
                    }
                    // PostgreSQL foreign key violation
                    if code == "23503" {
                        return DomainError::RepositoryError(
                            "Foreign key constraint violation".to_string(),
                        );
                    }
                }
                DomainError::RepositoryError(format!("Database error: {}", db_err))
            }
            _ => DomainError::RepositoryError(format!("Database operation failed: {}", error)),
        }
    }
}
