//! `gallery_group` — a logical group of gallery images.

use chrono::NaiveDateTime;
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "gallery_group")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub group_id: i32,
    pub name: String,
    pub cover_image: Option<String>,
    pub description: Option<String>,
    pub sort_order: Option<i32>,
    pub is_visible: Option<i8>,
    pub status: Option<i8>,
    pub create_time: Option<NaiveDateTime>,
    pub update_time: Option<NaiveDateTime>,
    pub create_by: Option<i32>,
    pub update_by: Option<i32>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}
impl ActiveModelBehavior for ActiveModel {}
