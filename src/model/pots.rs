use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Pots {
    pub pots: Vec<Pot>,
}

#[derive(Deserialize, Debug)]
pub struct Pot {
    pub id: String,
    pub name: String,
    pub balance: i64,
    pub currency: String,
    pub deleted: bool,
}

// -- Tests ---------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize_pots() {
        // Arrange
        let pots = r#"
            {
                "pots": [
                    {
                        "id": "pot_00009a9eY8Cw7ZyfZzZq8j",
                        "name": "Rainy Day Fund",
                        "balance": 1000,
                        "currency": "GBP",
                        "deleted": false
                    },
                    {
                        "id": "pot_00009a9eY8Cw7ZyfZzZq8j",
                        "name": "Holiday Fund",
                        "balance": 2000,
                        "currency": "GBP",
                        "deleted": false
                    }
                ]
            }
        "#;

        // Act
        let pots: Pots = serde_json::from_str(pots).unwrap();

        // Assert
        assert_eq!(pots.pots.len(), 2);
        assert_eq!(pots.pots[0].id, "pot_00009a9eY8Cw7ZyfZzZq8j");
        assert_eq!(pots.pots[0].name, "Rainy Day Fund");
        assert_eq!(pots.pots[0].balance, 1000);
        assert_eq!(pots.pots[0].currency, "GBP");
        assert_eq!(pots.pots[0].deleted, false);
    }
}
