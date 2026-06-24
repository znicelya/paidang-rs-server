//! `package` — photography package/product.

use chrono::NaiveDateTime;
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "package")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub package_id: i32,
    pub name: String,
    pub subtitle: Option<String>,
    pub category: Option<String>,
    pub price: i32,
    pub original_price: Option<i32>,
    pub deposit: Option<i32>,
    pub cover_image: Option<String>,
    pub description: Option<String>,
    pub service_items: Option<serde_json::Value>,
    pub suitable_people: Option<String>,
    pub shooting_location: Option<String>,
    pub validity_days: Option<i32>,
    pub sort_order: Option<i32>,
    pub is_hot: Option<i8>,
    pub is_recommend: Option<i8>,
    pub status: Option<i8>,
    pub view_count: Option<i32>,
    pub sale_count: Option<i32>,
    pub create_time: Option<NaiveDateTime>,
    pub update_time: Option<NaiveDateTime>,
    pub create_by: Option<i32>,
    pub update_by: Option<i32>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}
impl ActiveModelBehavior for ActiveModel {}
