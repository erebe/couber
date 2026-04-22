use base64::Engine;
use serde::{Deserialize, Serialize};
use serde_with::{json::JsonString, serde_as};
use std::path::Path;

const OPENROUTER_API_URL: &str = "https://openrouter.ai/api/v1/chat/completions";

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

/// Reads an image from `image_path`, sends it to the OpenRouter vision API,
/// and returns up to 50 tags/features that best describe the image.
///
/// Requires the `OPENROUTER_API_KEY` environment variable to be set.
/// The model used can be overridden via `OPENROUTER_MODEL` (default: `google/gemini-2.0-flash-001`).
pub async fn extract_image_tags(image_path: &Path) -> eyre::Result<Vec<String>> {
    let api_key = std::env::var("OPENROUTER_API_KEY")
        .map_err(|_| eyre::eyre!("OPENROUTER_API_KEY environment variable is not set"))?;

    let model = std::env::var("OPENROUTER_MODEL").unwrap_or_else(|_| "openrouter/auto".to_string());

    let image_bytes = std::fs::read(image_path)?;
    let mime_type = mime_type_from_path(image_path);
    let b64 = base64::engine::general_purpose::STANDARD.encode(&image_bytes);
    let data_url = format!("data:{};base64,{}", mime_type, b64);

    let request = ChatRequest {
        model,
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

    let client = reqwest::Client::new();
    let response = client
        .post(OPENROUTER_API_URL)
        .bearer_auth(&api_key)
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

fn mime_type_from_path(path: &Path) -> &'static str {
    match path.extension().and_then(|e| e.to_str()) {
        Some("jpg") | Some("jpeg") => "image/jpeg",
        Some("png") => "image/png",
        Some("gif") => "image/gif",
        Some("webp") => "image/webp",
        _ => "image/jpeg",
    }
}
