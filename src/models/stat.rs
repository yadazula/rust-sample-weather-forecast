use askama::Template;
use serde::{Deserialize, Serialize};

#[derive(Template)]
#[template(path = "stats.html")]
pub struct StatsTemplate {
    pub cities: Vec<City>,
}

#[derive(sqlx::FromRow, Serialize, Deserialize, Debug, Clone)]
pub struct City {
    pub name: String,
    pub latitude: f64,
    pub longitude: f64,
}
