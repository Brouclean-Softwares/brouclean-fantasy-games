use serde::Deserialize;

pub mod blood_bowl;
pub mod sessions;
pub mod users;

#[derive(Deserialize, sqlx::FromRow, Clone)]
pub struct Id {
    pub id: i32,
}

#[derive(Deserialize, sqlx::FromRow, Clone)]
pub struct IsTrue {
    pub is_true: bool,
}

#[derive(Deserialize, sqlx::FromRow, Clone)]
pub struct IsOptionalTrue {
    pub is_optional_true: Option<bool>,
}

#[derive(Deserialize, sqlx::FromRow, Clone)]
pub struct Total {
    pub total: Option<i64>,
}
