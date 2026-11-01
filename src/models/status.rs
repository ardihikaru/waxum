use serde::Deserialize;
use utoipa::ToSchema;

#[derive(Debug, Deserialize, ToSchema)]
pub struct StatusReactionRequest {
    #[schema(example = "559999999999@s.whatsapp.net")]
    pub status_owner: String,
    #[schema(example = "3EB0C431C26D15B441EE15")]
    pub message_id: String,
    #[schema(example = "\u{1F49A}")]
    pub reaction: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct StatusTextRequest {
    #[schema(example = "Hello from my status!")]
    pub text: String,
    /// Background colour as 0xAARRGGBB (decimal). Defaults to 0xFF1E6E4F.
    #[schema(example = 4280184399u32)]
    pub background_argb: Option<u32>,
    /// JIDs allowed to see the status.
    #[schema(example = json!(["559999999999@s.whatsapp.net"]))]
    pub recipients: Vec<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct StatusMediaRequest {
    /// Image or video (same shape as the `/messages/image` / `/messages/video` media field).
    pub media: crate::models::messages::MediaData,
    pub caption: Option<String>,
    /// JIDs allowed to see the status.
    #[schema(example = json!(["559999999999@s.whatsapp.net"]))]
    pub recipients: Vec<String>,
}
