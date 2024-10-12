use askama::Template;
use serde::{Deserialize, Serialize};

#[derive(Template)]
#[template(path = "index.html")]
pub struct IndexTemplate;

#[derive(Serialize, Deserialize)]
pub struct WeatherQuery {
    pub city: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct WeatherResponse {
    pub latitude: f64,
    pub longitude: f64,
    pub timezone: String,
    pub hourly: Hourly,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Hourly {
    pub time: Vec<String>,
    pub temperature_2m: Vec<f64>,
}

#[derive(Template, Deserialize, Debug)]
#[template(path = "weather.html")]
pub struct WeatherDisplayTemplate {
    city: String,
    forecasts: Vec<Forecast>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Forecast {
    pub date: String,
    pub temperature: String,
}

impl WeatherDisplayTemplate {
    /// Create a new `WeatherDisplay` from a `WeatherResponse`.
    pub(crate) fn new(city: String, response: WeatherResponse) -> Self {
        let display = WeatherDisplayTemplate {
            city,
            forecasts: response
                .hourly
                .time
                .iter()
                .zip(response.hourly.temperature_2m.iter())
                .map(|(date, temperature)| Forecast {
                    date: date.to_string(),
                    temperature: temperature.to_string(),
                })
                .collect(),
        };
        display
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GeoResponse {
    pub results: Vec<LatLong>,
}

#[derive(sqlx::FromRow, Serialize, Deserialize, Debug, Clone)]
pub struct LatLong {
    pub latitude: f64,
    pub longitude: f64,
}

#[derive(sqlx::FromRow, Serialize, Deserialize, Debug, Clone)]
pub struct City {
    pub name: String,
    pub latitude: f64,
    pub longitude: f64,
}

#[derive(Template)]
#[template(path = "stats.html")]
pub struct StatsTemplate {
    pub cities: Vec<City>,
}