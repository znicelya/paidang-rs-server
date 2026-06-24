//! `package_gallery` — sample images attached to a package.

use chrono::NaiveDateTime;
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "package_gallery")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub gallery_id: i32,
    pub package_id: i32,
    pub image_url: String,
    pub image_type: Option<String>,
    pub caption: Option<String>,
    pub sort_order: Option<i32>,
    pub create_time: Option<NaiveDateTime>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}
impl ActiveModelBehavior for ActiveModel {}
