use reqwest::{multipart, Client};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fmt;

// Custom error type for ImgBB operations
#[derive(Debug)]
pub enum ImgBBError {
    RequestError(reqwest::Error),
    ParseError(String),
}

impl fmt::Display for ImgBBError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ImgBBError::RequestError(err) => write!(f, "Request error: {}", err),
            ImgBBError::ParseError(err) => write!(f, "Parse error: {}", err),
        }
    }
}

impl Error for ImgBBError {}

// Conversion from reqwest::Error to our custom error
impl From<reqwest::Error> for ImgBBError {
    fn from(err: reqwest::Error) -> Self {
        ImgBBError::RequestError(err)
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ImgBBResponse {
    pub data: ImgBBData,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ImgBBData {
    pub id: String,
    pub url: String,
    pub display_url: String,
}

pub async fn upload_image(
    image_data: Vec<u8>,
    filename: String,
    api_key: &str,
) -> Result<String, ImgBBError> {
    let client = Client::new();
    let link = format!("https://api.imgbb.com/1/upload?key={}", api_key);

    let part = multipart::Part::bytes(image_data)
        .file_name(filename)
        .mime_str("image/png")
        .unwrap();

    let form_multipart = multipart::Form::new().part("image", part);

    let response = client.post(&link).multipart(form_multipart).send().await?; // This will convert reqwest::Error to ImgBBError

    if response.status().is_success() {
        let body = response.text().await?;

        // Parse the JSON, handling errors by mapping to our custom error
        let imgbb_response: ImgBBResponse =
            serde_json::from_str(&body).map_err(|e| ImgBBError::ParseError(e.to_string()))?;

        println!("{}", imgbb_response.data.url);
        Ok(imgbb_response.data.url)
    } else {
        Err(ImgBBError::from(response.error_for_status().err().unwrap()))
    }
}
