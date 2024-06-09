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
