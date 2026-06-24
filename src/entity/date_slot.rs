//! `date_slot` — concrete date-time instances.

use chrono::NaiveDateTime;
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "date_slot")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub slot_instance_id: i32,
    pub photographer_id: i32,
    pub template_id: Option<i32>,
    pub slot_date: String,
    pub slot_name: String,
    pub start_time: String,
    pub end_time: String,
    pub is_booked: Option<i8>,
    pub booking_id: Option<i32>,
    pub is_special: Option<i8>,
    pub status: Option<i8>,
    pub price: Option<i32>,
    pub remark: Option<String>,
    pub create_time: Option<NaiveDateTime>,
    pub update_time: Option<NaiveDateTime>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}
impl ActiveModelBehavior for ActiveModel {}
