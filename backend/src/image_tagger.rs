use base64::Engine;
use serde::{Deserialize, Serialize};
use serde_with::{json::JsonString, serde_as};
use std::path::Path;
use std::time::Duration;

#[derive(Serialize)]
struct ImageUrl {
    url: String,
}

#[derive(Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum ContentPart {
    Text { text: String },
    ImageUrl { image_url: ImageUrl },
}

#[derive(Serialize)]
struct Message {
    role: String,
    content: Vec<ContentPart>,
}

#[derive(Serialize)]
struct ChatRequest {
    model: String,
    stream: bool,
    messages: Vec<Message>,
}

#[derive(Deserialize)]
struct Choice {
    message: ResponseMessage,
}

#[serde_as]
#[derive(Deserialize)]
struct ResponseMessage {
    #[serde_as(as = "JsonString")]
    content: Vec<String>,
}

#[derive(Deserialize)]
struct ChatResponse {
    choices: Vec<Choice>,
}

pub struct ImageTaggerService {
    api_key: String,
    model: String,
    api_url: String,
    client: reqwest::Client,
}

impl ImageTaggerService {
    pub fn new(api_key: String, model: String, api_url: String) -> Self {
        let client = reqwest::Client::builder()
            .pool_max_idle_per_host(2)
            .connect_timeout(Duration::from_secs(10))
            .timeout(Duration::from_secs(60))
            .gzip(true)
            .zstd(true)
            .user_agent("couber/1.0")
            .build()
            .expect("Failed to build reqwest client");
        Self {
            api_key,
            model,
            api_url,
            client,
        }
    }

    /// Reads an image from `image_path`, sends it to the OpenRouter vision API,
    /// and returns up to 50 tags/features that best describe the image.
    pub async fn extract_image_tags(&self, image_path: &Path) -> eyre::Result<Vec<String>> {
        let image_bytes = std::fs::read(image_path)?;
        let mime_type = mime_type_from_path(image_path);
        let b64 = base64::engine::general_purpose::STANDARD.encode(&image_bytes);
        let data_url = format!("data:{};base64,{}", mime_type, b64);

        let request = ChatRequest {
            model: self.model.clone(),
            stream: false,
            messages: vec![Message {
                role: "user".to_string(),
                content: vec![
                    ContentPart::Text {
                        text: "Analyze this image and return exactly 50 tags or features that best describe it. \
                               Output ONLY a JSON array of strings, no explanation, no markdown, no code block. \
                               Example: [\"tag1\", \"tag2\", ...]".to_string(),
                    },
                    ContentPart::ImageUrl {
                        image_url: ImageUrl { url: data_url },
                    },
                ],
            }],
        };

        let response = self
            .client
            .post(&self.api_url)
            .bearer_auth(&self.api_key)
            .json(&request)
            .send()
            .await?
            .error_for_status()?
            .json::<ChatResponse>()
            .await?;

        let tags = response
            .choices
            .into_iter()
            .next()
            .ok_or_else(|| eyre::eyre!("No choices returned from API"))?
            .message
            .content;

        Ok(tags)
    }
}

fn mime_type_from_path(path: &Path) -> &'static str {
    match path.extension().and_then(|e| e.to_str()) {
        Some("jpg") | Some("jpeg") => "image/jpeg",
        Some("png") => "image/png",
        Some("gif") => "image/gif",
        Some("webp") => "image/webp",
        _ => "image/jpeg",
    }
}
