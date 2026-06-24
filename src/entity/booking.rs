//! `booking` — reservation.

use chrono::NaiveDateTime;
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "booking")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub booking_id: i32,
    pub booking_no: String,
    pub user_id: Option<i32>,
    pub photographer_id: i32,
    pub slot_instance_id: Option<i32>,
    pub package_id: Option<i32>,
    pub booking_date: String,
    pub start_time: String,
    pub end_time: String,
    pub total_amount: Option<i32>,
    pub deposit_amount: Option<i32>,
    pub paid_amount: Option<i32>,
    pub status: Option<String>,
    pub cancel_reason: Option<String>,
    pub cancel_time: Option<NaiveDateTime>,
    pub customer_name: String,
    pub customer_phone: String,
    pub customer_remark: Option<String>,
    pub photographer_remark: Option<String>,
    pub reminder_sent: Option<i8>,
    pub create_time: Option<NaiveDateTime>,
    pub update_time: Option<NaiveDateTime>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}
impl ActiveModelBehavior for ActiveModel {}
