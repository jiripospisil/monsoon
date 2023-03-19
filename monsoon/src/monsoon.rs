use chrono::{DateTime, FixedOffset};
use tower_service::Service;

use std::{
    borrow::Cow,
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

use crate::{body::Body, client::Client, Error, Result};

/// The coordinates for which the weather should be looked up.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct Params {
    pub lat: f64,
    pub lon: f64,
    pub alt: Option<i32>,

    pub last_response: Option<Response>,
}

impl Params {
    /// Creates a new Params instance. The individual parameters are validated
    /// and normalized (truncated to 4 franctional places) based on the APIs
    /// requirements.
    ///
    /// Example:
    ///
    /// ```no_run
    /// use monsoon::Params;
    ///
    /// # #[tokio::main]
    /// # async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    /// let params = Params::new(50.0880, 14.4207, 320)?;
    /// # Ok(())
    /// # }
    ///```
    pub fn new(lat: f64, lon: f64, alt: impl Into<Option<i32>>) -> Result<Self> {
        Self::new_with_last_response(lat, lon, alt, None)
    }

    pub fn new_with_last_response(
        lat: f64,
        lon: f64,
        alt: impl Into<Option<i32>>,
        last_response: impl Into<Option<Response>>,
    ) -> Result<Self> {
        if !lat.is_finite() || lat.abs() > 90.0 {
            return Err(Error::Params("Invalid lat value."));
        }

        if !lon.is_finite() || lon.abs() > 180.0 {
            return Err(Error::Params("Invalid lon value."));
        }

        let alt = alt.into();
        if let Some(alt) = alt {
            if !(-500..=9000).contains(&alt) {
                return Err(Error::Params("Invalid alt value."));
            }
        }

        Ok(Self {
            lat: (lat * 10000.0).trunc() / 10000.0,
            lon: (lon * 10000.0).trunc() / 10000.0,
            alt,
            last_response: last_response.into(),
        })
    }
}

/// Response from the API.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct Response {
    expires_at: DateTime<FixedOffset>,
    last_modified: Box<str>,
    pub(crate) raw_body: Box<str>,
}

impl Response {
    pub(crate) fn new(
        expires_at: DateTime<FixedOffset>,
        last_modified: Box<str>,
        raw_body: Box<str>,
    ) -> Self {
        Self {
            expires_at,
            last_modified,
            raw_body,
        }
    }

    pub fn expires_at(&self) -> &DateTime<FixedOffset> {
        &self.expires_at
    }

    pub fn last_modified(&self) -> &str {
        &self.last_modified
    }

    pub fn body(&self) -> Result<Body<'_>> {
        serde_json::from_str::<Body>(&self.raw_body).map_err(Into::into)
    }
}

/// The main entry point of the library.
#[derive(Debug, Clone)]
pub struct Monsoon {
    client: Client,
}

impl Monsoon {
    /// Creates a new instance with the given user agent.
    ///
    /// Example:
    ///
    ///```no_run
    ///use monsoon::Monsoon;
    ///
    ///let monsoon = Monsoon::new("test.com support@test.com");
    ///```
    pub fn new(user_agent: impl Into<Cow<'static, str>>) -> Result<Self> {
        let client = Client::new(user_agent.into())?;
        Ok(Self { client })
    }

    /// Fetches weather data for the given coordinates.
    ///
    /// Example:
    ///
    /// ```no_run
    /// use monsoon::Monsoon;
    ///
    /// # #[tokio::main]
    /// # async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    /// let monsoon = Monsoon::new("test.com support@test.com")?;
    /// let response = monsoon.get(50.0880, 14.4207).await?;
    /// let body = response.body()?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get(&self, lat: f64, lon: f64) -> Result<Response> {
        self.get_with_params(Params::new(lat, lon, None)?).await
    }

    /// Fetches weather data for the given coordinates including altitude.
    ///
    /// Example:
    ///
    /// ```no_run
    /// use monsoon::Monsoon;
    ///
    /// # #[tokio::main]
    /// # async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    /// let monsoon = Monsoon::new("test.com support@test.com")?;
    /// let response = monsoon.get_with_altitude(50.0880, 14.4207, 345).await?;
    /// let body = response.body()?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_with_altitude(&self, lat: f64, lon: f64, alt: i32) -> Result<Response> {
        self.get_with_params(Params::new(lat, lon, alt)?).await
    }

    /// Fetches weather data for the given coordinates provided via Params.
    ///
    /// Example:
    ///
    /// ```no_run
    /// use monsoon::{Monsoon, Params};
    ///
    /// # #[tokio::main]
    /// # async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    /// let params =  Params::new(50.0880, 14.4207, Some(345))?;
    ///
    /// let monsoon = Monsoon::new("test.com support@test.com")?;
    /// let response = monsoon.get_with_params(params).await?;
    /// let body = response.body()?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_with_params(&self, params: Params) -> Result<Response> {
        self.client.get(params).await
    }
}

impl Service<Params> for Monsoon {
    type Response = Response;
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response>> + Send>>;

    fn poll_ready(&mut self, _: &mut Context<'_>) -> Poll<std::result::Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, params: Params) -> Self::Future {
        let clone = self.clone();
        Box::pin(async move { clone.get_with_params(params).await })
    }
}

#[cfg(test)]
mod tests {
    mod params {
        use crate::Params;

        #[test]
        fn validates_lat_value() {
            for lat in [f64::INFINITY, f64::NEG_INFINITY, -91.0, 91.0] {
                assert!(Params::new(lat, 100.0, None).is_err());
            }

            for lat in [-90.0, 90.0, 42.0] {
                assert!(Params::new(lat, 100.0, None).is_ok());
            }
        }

        #[test]
        fn validates_lon_value() {
            for lat in [f64::INFINITY, f64::NEG_INFINITY, -181.0, 181.0] {
                assert!(Params::new(50.0, lat, None).is_err());
            }

            for lat in [-180.0, 180.0, 42.0] {
                assert!(Params::new(50.0, lat, None).is_ok());
            }
        }

        #[test]
        fn validates_alt_value() {
            for alt in [-501, 9001] {
                assert!(Params::new(50.0, 42.0, alt).is_err());
            }

            for alt in [-500, 9000, 42] {
                assert!(Params::new(50.0, 42.0, alt).is_ok());
            }
        }

        #[test]
        fn truncates_values() {
            let params = Params::new(14.1234567, 12.7654321, None).unwrap();
            assert_eq!(params.lat, 14.1234);
            assert_eq!(params.lon, 12.7654);
        }
    }
}
