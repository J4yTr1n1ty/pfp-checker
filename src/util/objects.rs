use serde::{Deserialize, Serialize};

pub struct ProfilePictureEntry {
    pub title: String,
    pub content: String,
    pub inline: bool,
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
