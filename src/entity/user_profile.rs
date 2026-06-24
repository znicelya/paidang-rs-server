//! `user_profile` table entity. Matches migration m20250101_0001_users_profiles.
//! No FK to `user` — referential integrity is enforced in the service layer.

use chrono::NaiveDateTime;
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "user_profile")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub profile_id: i32,
    /// UNIQUE; points to `user.user_id` (app-level, no FK)
    pub user_id: i32,
    pub nickname: Option<String>,
    pub avatar_url: Option<String>,
    pub background_image: Option<String>,
    /// 0=未知, 1=男, 2=女
    pub gender: Option<i8>,
    pub country: Option<String>,
    pub province: Option<String>,
    pub city: Option<String>,
    pub birthday: Option<String>,
    pub bio: Option<String>,
    pub create_time: Option<NaiveDateTime>,
    pub update_time: Option<NaiveDateTime>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::user::Entity",
        from = "Column::UserId",
        to = "super::user::Column::UserId"
    )]
    User,
}

impl Related<super::user::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::User.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
