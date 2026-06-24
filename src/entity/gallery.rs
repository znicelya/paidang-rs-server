//! `gallery` — individual gallery image/media entry.

use chrono::NaiveDateTime;
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "gallery")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub gallery_id: i32,
    pub group_id: Option<i32>,
    pub title: String,
    pub subtitle: Option<String>,
    pub cover_image: Option<String>,
    pub image_url: Option<String>,
    pub image_list: Option<serde_json::Value>,
    pub video_url: Option<String>,
    pub media_type: Option<String>,
    pub tags: Option<String>,
    pub photographer_id: Option<i32>,
    pub photographer_name: Option<String>,
    pub shooting_location: Option<String>,
    pub shooting_date: Option<String>,
    pub width: Option<i32>,
    pub height: Option<i32>,
    pub file_size: Option<i32>,
    pub view_count: Option<i32>,
    pub like_count: Option<i32>,
    pub sort_order: Option<i32>,
    pub is_cover: Option<i8>,
    pub status: Option<i8>,
    pub create_time: Option<NaiveDateTime>,
    pub update_time: Option<NaiveDateTime>,
    pub create_by: Option<i32>,
    pub update_by: Option<i32>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}
impl ActiveModelBehavior for ActiveModel {}
