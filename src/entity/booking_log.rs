//! `booking_log` — reservation action audit.

use chrono::NaiveDateTime;
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "booking_log")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub log_id: i32,
    pub booking_id: i32,
    pub action: String,
    pub from_status: Option<String>,
    pub to_status: Option<String>,
    pub operator_id: Option<i32>,
    pub operator_type: Option<String>,
    pub remark: Option<String>,
    pub create_time: Option<NaiveDateTime>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}
impl ActiveModelBehavior for ActiveModel {}
