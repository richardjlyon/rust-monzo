use serde::Deserialize;

#[derive(Deserialize, Debug, Default)]
pub struct Balance {
    pub balance: i64,
    pub total_balance: i64,
    pub currency: String,
    pub spend_today: i64,
}

// -- Tests -------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize_balance() {
        let json = r#"{
            "balance": 1000,
            "total_balance": 1000,
            "currency": "GBP",
            "spend_today": 0
        }"#;

        let balance: Balance = serde_json::from_str(json).unwrap();
        assert_eq!(balance.balance, 1000);
        assert_eq!(balance.total_balance, 1000);
        assert_eq!(balance.currency, "GBP");
        assert_eq!(balance.spend_today, 0);
    }
}
