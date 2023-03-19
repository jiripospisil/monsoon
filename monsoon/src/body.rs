use chrono::{DateTime, Utc};
use serde::Deserialize;

/// Response body from the "complete" API as defined in the [`documentation`]. Head over there to
/// learn more about the individual fields if necessary.
///
/// [`documentation`]: https://api.met.no/weatherapi/locationforecast/2.0/documentation
#[derive(Debug, PartialEq, Deserialize)]
pub struct Body<'a> {
    #[serde(rename(deserialize = "type"))]
    pub type_field: &'a str,
    pub geometry: Geometry<'a>,
    pub properties: Properties<'a>,
}

#[derive(Debug, PartialEq, Deserialize)]
pub struct Geometry<'a> {
    #[serde(rename(deserialize = "type"))]
    pub type_field: &'a str,
    pub coordinates: Coordinates,
}

#[derive(Debug, PartialEq, Deserialize)]
pub struct Coordinates {
    pub longitude: f64,
    pub latitude: f64,
    pub altitude: f64,
}

#[derive(Debug, PartialEq, Deserialize)]
#[serde(bound(deserialize = "'de: 'a"))]
pub struct Properties<'a> {
    pub meta: Meta<'a>,
    pub timeseries: Box<[TimeSeries<'a>]>,
}

#[derive(Debug, PartialEq, Deserialize)]
#[serde(bound(deserialize = "'de: 'a"))]
pub struct Meta<'a> {
    pub updated_at: DateTime<Utc>,
    pub units: Units<'a>,
}

#[derive(Debug, PartialEq, Deserialize)]
pub struct Units<'a> {
    pub air_pressure_at_sea_level: Option<&'a str>,
    pub air_temperature: Option<&'a str>,
    pub air_temperature_max: Option<&'a str>,
    pub air_temperature_min: Option<&'a str>,
    pub cloud_area_fraction: Option<&'a str>,
    pub cloud_area_fraction_high: Option<&'a str>,
    pub cloud_area_fraction_low: Option<&'a str>,
    pub cloud_area_fraction_medium: Option<&'a str>,
    pub dew_point_temperature: Option<&'a str>,
    pub fog_area_fraction: Option<&'a str>,
    pub precipitation_amount: Option<&'a str>,
    pub relative_humidity: Option<&'a str>,
    pub ultraviolet_index_clear_sky: Option<&'a str>,
    pub wind_from_direction: Option<&'a str>,
    pub wind_speed: Option<&'a str>,
}

#[derive(Debug, PartialEq, Deserialize)]
#[serde(bound(deserialize = "'de: 'a"))]
pub struct TimeSeries<'a> {
    pub time: DateTime<Utc>,
    pub data: Data<'a>,
}

#[derive(Debug, PartialEq, Deserialize)]
#[serde(bound(deserialize = "'de: 'a"))]
pub struct Data<'a> {
    pub instant: Instant,
    pub next_12_hours: Option<NextHours<'a>>,
    pub next_1_hours: Option<NextHours<'a>>,
    pub next_6_hours: Option<NextHours<'a>>,
}

#[derive(Debug, PartialEq, Deserialize)]
pub struct Instant {
    pub details: InstantDetails,
}

#[derive(Debug, PartialEq, Deserialize)]
pub struct InstantDetails {
    pub air_pressure_at_sea_level: Option<f64>,
    pub air_temperature: Option<f64>,
    pub cloud_area_fraction: Option<f64>,
    pub cloud_area_fraction_high: Option<f64>,
    pub cloud_area_fraction_low: Option<f64>,
    pub cloud_area_fraction_medium: Option<f64>,
    pub dew_point_temperature: Option<f64>,
    pub fog_area_fraction: Option<f64>,
    pub relative_humidity: Option<f64>,
    pub ultraviolet_index_clear_sky: Option<f64>,
    pub wind_from_direction: Option<f64>,
    pub wind_speed: Option<f64>,
}

#[derive(Debug, PartialEq, Deserialize)]
#[serde(bound(deserialize = "'de: 'a"))]
pub struct NextHours<'a> {
    // Not optional in the docs but the API doesn't return it in all cases.
    pub details: Option<SummaryDetails>,
    pub summary: Summary<'a>,
}

#[derive(Debug, PartialEq, Deserialize)]
pub struct SummaryDetails {
    pub air_temperature_max: Option<f64>,
    pub air_temperature_min: Option<f64>,
    pub precipitation_amount: Option<f64>,
    pub precipitation_amount_max: Option<f64>,
    pub precipitation_amount_min: Option<f64>,
    pub probability_of_precipitation: Option<f64>,
    pub probability_of_thunder: Option<f64>,
    pub ultraviolet_index_clear_sky_max: Option<f64>,
}

#[derive(Debug, PartialEq, Deserialize)]
pub struct Summary<'a> {
    pub symbol_code: &'a str,
}
