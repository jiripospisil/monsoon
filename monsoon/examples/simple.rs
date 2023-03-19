use cli_table::{format::Justify, print_stdout, Cell, CellStruct, Style, Table};

use monsoon::Monsoon;
use std::{error::Error, result::Result};

fn to_cell(val: Option<f64>) -> CellStruct {
    val.map(|val| val.to_string())
        .unwrap_or(String::from("??"))
        .cell()
        .justify(Justify::Center)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let monsoon = Monsoon::new("test.com support@test.com")?;

    // Prague
    let response = monsoon.get(50.0880, 14.4207).await?;
    let body = response.body()?;

    let table = body
        .properties
        .timeseries
        .iter()
        .map(|time_series| {
            let details = &time_series.data.instant.details;
            vec![
                time_series.time.to_rfc2822().cell(),
                to_cell(details.air_temperature),
                to_cell(details.wind_speed),
                to_cell(details.ultraviolet_index_clear_sky),
            ]
        })
        .collect::<Vec<_>>()
        .table()
        .title(vec![
            "Date & Time".cell().bold(true),
            format!(
                "Air Temperature ({})",
                body.properties.meta.units.air_temperature.unwrap_or("??")
            )
            .cell()
            .bold(true),
            format!(
                "Wind Speed ({})",
                body.properties.meta.units.wind_speed.unwrap_or("??")
            )
            .cell()
            .bold(true),
            "UV Index".cell().bold(true),
        ]);

    print_stdout(table).unwrap();

    Ok(())
}
