//! `time_slot_template` — photographer time slot templates.

use chrono::NaiveDateTime;
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "time_slot_template")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub template_id: i32,
    pub photographer_id: i32,
    pub slot_name: String,
    pub start_time: String,
    pub end_time: String,
    pub sort_order: Option<i32>,
    pub is_default: Option<i8>,
    pub status: Option<i8>,
    pub create_time: Option<NaiveDateTime>,
    pub update_time: Option<NaiveDateTime>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}
impl ActiveModelBehavior for ActiveModel {}
