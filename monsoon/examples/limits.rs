use std::{error::Error, time::Duration};

use monsoon::{Monsoon, Params};
use tower::{Service, ServiceBuilder, ServiceExt};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let monsoon = Monsoon::new("test.com support@test.com")?;

    let mut service = ServiceBuilder::new()
        // At most 50 requests in-flight at the same time
        .concurrency_limit(50)
        // At most 20 requests per second
        .rate_limit(20, Duration::from_secs(1))
        .service(monsoon);

    let response = service
        .ready()
        .await?
        .call(Params::new(50.0880, 14.4207, None)?)
        .await?;
    let body = response.body()?;

    dbg!(body.geometry);

    Ok(())
}
