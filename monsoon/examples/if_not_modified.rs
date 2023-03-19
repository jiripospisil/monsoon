use monsoon::{Monsoon, Params};
use std::{error::Error, result::Result, time::Instant};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let monsoon = Monsoon::new("test.com support@test.com")?;

    // Get the first response
    let now = Instant::now();
    let response = monsoon.get(50.0880, 14.4207).await?;
    println!("The first request took: {} ms", now.elapsed().as_millis());

    // Pass in the previous response
    let now = Instant::now();
    _ = monsoon
        .get_with_params(Params::new_with_last_response(
            50.0880, 14.4207, None, response,
        )?)
        .await?;
    println!(
        "The second cached request took: {} ms",
        now.elapsed().as_millis()
    );

    Ok(())
}
