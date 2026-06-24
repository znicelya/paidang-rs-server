//! `date_setting` — per-date availability and blocks.

use chrono::NaiveDateTime;
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "date_setting")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub setting_id: i32,
    pub photographer_id: i32,
    pub target_date: String,
    pub start_time: Option<String>,
    pub end_time: Option<String>,
    pub is_available: Option<i8>,
    pub use_template_id: Option<i32>,
    pub reason: Option<String>,
    pub create_time: Option<NaiveDateTime>,
    pub update_time: Option<NaiveDateTime>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}
impl ActiveModelBehavior for ActiveModel {}
