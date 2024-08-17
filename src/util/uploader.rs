use reqwest::{multipart, Client};

use super::objects::ImgBBResponse;

pub async fn upload_image_to_img_bb(
    image_data: Vec<u8>,
    user_id: i64,
) -> Result<String, reqwest::Error> {
    let api_key = std::env::var("IMGBB_KEY").expect("ImgBB API Key must be set.");

    let client = Client::new();

    let link = format!("https://api.imgbb.com/1/upload?key={}", api_key);

    let timestamp = chrono::Utc::now().timestamp();

    let part = multipart::Part::bytes(image_data)
        .file_name(format!("pfp_{}_{}.png", user_id, timestamp))
        .mime_str("image/png")
        .unwrap();

    let form_multipart = multipart::Form::new().part("image", part);

    let response = client
        .post(&link)
        .multipart(form_multipart)
        .send()
        .await
        .unwrap();

    if response.status().is_success() {
        let body = response.text().await.unwrap();
        let imgbb_response: ImgBBResponse = serde_json::from_str(&body).unwrap();
        Ok(imgbb_response.data.url)
    } else {
        Err(response.error_for_status().err().unwrap())
    }
}
