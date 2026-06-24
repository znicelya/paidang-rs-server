//! `package_item` — line items inside a package.

use chrono::NaiveDateTime;
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "package_item")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub item_id: i32,
    pub package_id: i32,
    pub item_type: String,
    pub item_name: String,
    pub quantity: Option<i32>,
    pub unit: Option<String>,
    pub item_value: Option<String>,
    pub sort_order: Option<i32>,
    pub is_default: Option<i8>,
    pub create_time: Option<NaiveDateTime>,
    pub update_time: Option<NaiveDateTime>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}
impl ActiveModelBehavior for ActiveModel {}
