//! Monsoon is a library for accessing weather data produced by [The Norwegian Meteorological
//! Institute]. Most notably, this data is used on [Yr.no].
//!
//! Example:
//!
//! ```no_run
//! use std::error::Error;
//! use monsoon::Monsoon;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn Error>> {
//!   let monsoon = Monsoon::new("test.com support@test.com")?;
//!   let response = monsoon.get(50.0880, 14.4207).await?;
//!   let body = response.body()?;
//!   dbg!(body);
//!
//!   Ok(())
//! }
//! ```
//!
//! You're required to properly identify yourself. In this case, the string `"test.com
//! support@test.com"` will be sent in the `User-Agent` of every request.
//!
//! You're further required to a rate limit of 20 requests per second and to respect the "Expires"
//! header of each response. Monsoon doesn't implement these rules on its own but it does
//! implement the [Service] trait of [Tower] and as such you can use middleware in the Tower
//! ecosystem to implement them. See [Examples]. Finally, see the [Terms of Service] for more information.
//!
//! [The Norwegian Meteorological Institute]: https://www.met.no/en
//! [Yr.no]: https://www.yr.no/en
//! [Service]: https://docs.rs/tower-service/latest/tower_service/trait.Service.html
//! [Tower]: https://docs.rs/tower/latest/tower
//! [Examples]: https://github.com/jiripospisil/monsoon/tree/master/monsoon/examples
//! [Terms of Service]: https://api.met.no/doc/TermsOfService
pub mod body;
mod client;
mod error;
mod monsoon;

pub use crate::monsoon::{Monsoon, Params, Response};
pub use error::{Error, Result};
