use std::borrow::Cow;

use chrono::{DateTime, FixedOffset, Utc};
use reqwest::{
    header::{HeaderMap, HeaderValue, IF_MODIFIED_SINCE},
    StatusCode, Url,
};

use crate::{Error, Params, Response, Result};

#[derive(Debug, Clone)]
pub struct Client {
    client: reqwest::Client,
}

impl Client {
    pub fn new(user_agent: Cow<'static, str>) -> Result<Self> {
        let client = reqwest::Client::builder()
            .user_agent(user_agent.as_ref())
            .build()
            .map_err(generalize_error)?;

        Ok(Self { client })
    }

    pub async fn get(&self, params: Params) -> Result<Response> {
        if let Some(last_response) = &params.last_response {
            if last_response.expires_at() > &Utc::now() {
                return Ok(params.last_response.unwrap());
            }
        }

        self.get_from_api(params).await
    }

    async fn get_from_api(&self, params: Params) -> Result<Response> {
        let response = {
            let url = create_url(&params);
            let headers = create_headers(&params)?;

            self.client
                .get(url)
                .headers(headers)
                .send()
                .await
                .map_err(generalize_error)?
        };

        match response.error_for_status() {
            Ok(response) => match response.status() {
                StatusCode::OK => handle_ok_response(response).await,
                StatusCode::NOT_MODIFIED => handle_not_modified_response(params, response).await,
                StatusCode::TOO_MANY_REQUESTS => {
                    Err(Error::Response("Too many requests (HTTP 429)".into()))
                }
                code => Err(Error::Response(
                    format!("Unexpected error code: {}", code).into(),
                )),
            },
            Err(err) => Err(Error::Response(err.to_string().into())),
        }
    }
}

fn create_url(params: &Params) -> Url {
    let mut url = Url::parse_with_params(
        "https://api.met.no/weatherapi/locationforecast/2.0/complete",
        &[
            ("lat", params.lat.to_string()),
            ("lon", params.lon.to_string()),
        ],
    )
    .expect("valid URL");

    if let Some(alt) = params.alt {
        url.query_pairs_mut()
            .append_pair("altitude", &alt.to_string());
    }

    url
}

fn create_headers(params: &Params) -> Result<HeaderMap> {
    let mut map = HeaderMap::new();

    if let Some(last_response) = &params.last_response {
        map.append(
            IF_MODIFIED_SINCE,
            HeaderValue::from_str(last_response.last_modified()).map_err(|_| {
                Error::Request("Unable to serialize a valid last-modified value.".into())
            })?,
        );
    }

    Ok(map)
}

fn extract_headers(response: &reqwest::Response) -> Result<(DateTime<FixedOffset>, Box<str>)> {
    let expires_at = response
        .headers()
        .get("expires")
        .ok_or(Error::Response("Missing expires header".into()))?
        .to_str()
        .map_err(|_| Error::Response("Invalid expires header.".into()))?;

    let last_modified = response
        .headers()
        .get("last-modified")
        .ok_or(Error::Response("Missing last-modified header".into()))?
        .to_str()
        .map_err(|_| Error::Response("Invalid last-modified header.".into()))?
        .to_string();

    Ok((
        DateTime::parse_from_rfc2822(expires_at)
            .map_err(|_| Error::Response("Unable to parse expires header.".into()))?,
        last_modified.into_boxed_str(),
    ))
}

async fn handle_ok_response(response: reqwest::Response) -> Result<Response> {
    let (expires_at, last_modified) = extract_headers(&response)?;
    let raw_body = response
        .text()
        .await
        .map_err(|_| Error::Response("Failed to decode response.".into()))?
        .into_boxed_str();

    Ok(Response::new(expires_at, last_modified, raw_body))
}

async fn handle_not_modified_response(
    params: Params,
    response: reqwest::Response,
) -> Result<Response> {
    let (expires_at, last_modified) = extract_headers(&response)?;

    let last_response = params
        .last_response
        .expect("304 only with a valid last response");

    Ok(Response::new(
        expires_at,
        last_modified,
        last_response.raw_body,
    ))
}

fn generalize_error(err: impl ToString) -> Error {
    Error::HttpClient(err.to_string())
}
