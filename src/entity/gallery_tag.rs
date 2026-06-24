//! `gallery_tag` — tags used to categorize gallery content.

use chrono::NaiveDateTime;
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "gallery_tag")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub tag_id: i32,
    pub tag_name: String,
    pub tag_type: Option<String>,
    pub use_count: Option<i32>,
    pub sort_order: Option<i32>,
    pub create_time: Option<NaiveDateTime>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}
impl ActiveModelBehavior for ActiveModel {}
