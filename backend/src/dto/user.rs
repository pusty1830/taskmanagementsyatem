use serde::Serialize;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::domain::{Role, User};

#[derive(Debug, Serialize, ToSchema)]
pub struct UserDto {
    pub id: Uuid,
    #[schema(example = "James Bond")]
    pub full_name: String,
    #[schema(example = "jamesbond@example.com")]
    pub email: String,
    pub role: Role,
}

impl From<User> for UserDto {
    fn from(u: User) -> Self {
        Self {
            id: u.id,
            full_name: u.full_name,
            email: u.email,
            role: u.role,
        }
    }
}
