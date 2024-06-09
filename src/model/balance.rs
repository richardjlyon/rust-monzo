use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Balance {
    pub balance: i64,
    pub total_balance: i64,
    pub currency: String,
    pub spend_today: i64,
}
