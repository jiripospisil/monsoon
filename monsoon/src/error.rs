use std::borrow::Cow;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("An error occured while working with the Http client.")]
    HttpClient(String),

    #[error("Invalid or unexpected response.")]
    Response(Cow<'static, str>),

    #[error("An error while building a request.")]
    Request(Cow<'static, str>),

    #[error("Unable to deserialize the JSON body.")]
    ResponseBody(#[from] serde_json::Error),

    #[error("Invalid params provided.")]
    Params(&'static str),
}

pub type Result<T> = std::result::Result<T, Error>;
