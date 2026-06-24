//! `user` table entity. Matches migration m20250101_0001_users_profiles.

use chrono::NaiveDateTime;
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "user")]
pub struct Model {
    #[sea_orm(primary_key)]
    #[serde(rename = "user_id")]
    pub user_id: i32,
    pub openid: String,
    pub unionid: Option<String>,
    pub session_key: Option<String>,
    /// 0=普通用户, 1=摄影师, 2=管理员
    pub role: i8,
    pub phone: Option<String>,
    /// 0=禁用, 1=正常, 2=注销
    pub status: i8,
    pub last_login_time: Option<NaiveDateTime>,
    pub create_time: Option<NaiveDateTime>,
    pub update_time: Option<NaiveDateTime>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_one = "super::user_profile::Entity")]
    UserProfile,
}

impl Related<super::user_profile::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::UserProfile.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
