use axum::{
    extract::{Path, State},
    Json,
};
use wacore_binary::jid::Jid;
use waproto::buffa::MessageField;
use waproto::whatsapp as wa;

use crate::error::ApiError;
use crate::models::common::SuccessResponse;
use crate::models::messages::MessageResponse;
use crate::models::status::{StatusMediaRequest, StatusReactionRequest, StatusTextRequest};
use crate::state::AppState;

#[utoipa::path(
    post,
    security(("bearer_auth" = [])),
    path = "/api/v1/sessions/{session_id}/status/react",
    tag = "status",
    params(
        ("session_id" = String, Path, description = "Session ID")
    ),
    request_body = StatusReactionRequest,
    responses(
        (status = 200, description = "Status reaction sent", body = SuccessResponse),
        (status = 400, description = "Invalid JID"),
        (status = 404, description = "Session not found"),
        (status = 503, description = "Not connected")
    )
)]
pub async fn send_status_reaction(
    State(state): State<AppState>,
    Path(session_id): Path<String>,
    Json(request): Json<StatusReactionRequest>,
) -> Result<Json<SuccessResponse>, ApiError> {
    let client = get_client(&state, &session_id)?;
    let owner: Jid = request
        .status_owner
        .parse()
        .map_err(|_| ApiError::InvalidJid(request.status_owner.clone()))?;

    let status_broadcast = Jid::status_broadcast();
    let now_ms = chrono::Utc::now().timestamp_millis();

    let message = wa::Message {
        reaction_message: MessageField::some(wa::message::ReactionMessage {
            key: Some(wa::MessageKey {
                remote_jid: Some(status_broadcast.to_string()),
                from_me: Some(false),
                id: Some(request.message_id.clone()),
                participant: Some(owner.to_string()),
            })
            .into(),
            text: Some(request.reaction.clone()),
            grouping_key: None,
            sender_timestamp_ms: Some(now_ms),
        }),
        ..Default::default()
    };

    client
        .send_message(status_broadcast, message)
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    Ok(Json(SuccessResponse::with_message("Status reaction sent")))
}

fn get_client(
    state: &AppState,
    session_id: &str,
) -> Result<std::sync::Arc<whatsapp_rust::Client>, ApiError> {
    let runtime = state
        .get_session(session_id)
        .ok_or(ApiError::NotConnected)?;

    runtime.get_live_client().ok_or(ApiError::NotConnected)
}

fn parse_recipients(raw: &[String]) -> Result<Vec<Jid>, ApiError> {
    if raw.is_empty() {
        return Err(ApiError::BadRequest(
            "recipients must contain at least one JID".to_string(),
        ));
    }
    raw.iter()
        .map(|r| r.parse().map_err(|_| ApiError::InvalidJid(r.clone())))
        .collect()
}

fn status_response(message_id: String) -> Json<MessageResponse> {
    Json(MessageResponse {
        message_id,
        timestamp: chrono::Utc::now().timestamp(),
        to: Jid::status_broadcast().to_string(),
    })
}

#[utoipa::path(
    post,
    security(("bearer_auth" = [])),
    path = "/api/v1/sessions/{session_id}/status/text",
    tag = "status",
    params(("session_id" = String, Path, description = "Session ID")),
    request_body = StatusTextRequest,
    responses(
        (status = 200, description = "Status posted", body = MessageResponse),
        (status = 400, description = "Invalid request"),
        (status = 404, description = "Session not found"),
        (status = 503, description = "Not connected")
    )
)]
pub async fn post_status_text(
    State(state): State<AppState>,
    Path(session_id): Path<String>,
    Json(request): Json<StatusTextRequest>,
) -> Result<Json<MessageResponse>, ApiError> {
    let client = get_client(&state, &session_id)?;
    let recipients = parse_recipients(&request.recipients)?;
    let result = client
        .status()
        .send_text(
            &request.text,
            request.background_argb.unwrap_or(0xFF1E6E4F),
            wa::message::extended_text_message::FontType::SYSTEM,
            &recipients,
            Default::default(),
        )
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?;
    Ok(status_response(result.message_id))
}

#[utoipa::path(
    post,
    security(("bearer_auth" = [])),
    path = "/api/v1/sessions/{session_id}/status/image",
    tag = "status",
    params(("session_id" = String, Path, description = "Session ID")),
    request_body = StatusMediaRequest,
    responses(
        (status = 200, description = "Status posted", body = MessageResponse),
        (status = 400, description = "Invalid request"),
        (status = 404, description = "Session not found"),
        (status = 503, description = "Not connected")
    )
)]
pub async fn post_status_image(
    State(state): State<AppState>,
    Path(session_id): Path<String>,
    Json(request): Json<StatusMediaRequest>,
) -> Result<Json<MessageResponse>, ApiError> {
    let client = get_client(&state, &session_id)?;
    let recipients = parse_recipients(&request.recipients)?;
    let media = crate::handlers::messages::resolve_media(
        &client,
        &request.media,
        wacore::download::MediaType::Image,
    )
    .await?;
    let message = wa::Message {
        image_message: MessageField::some(wa::message::ImageMessage {
            url: Some(media.url),
            direct_path: Some(media.direct_path),
            media_key: Some(media.media_key),
            file_sha256: Some(media.file_sha256),
            file_enc_sha256: Some(media.file_enc_sha256),
            file_length: Some(media.file_length),
            mimetype: Some(media.mimetype),
            caption: request.caption,
            ..Default::default()
        }),
        ..Default::default()
    };
    let result = client
        .status()
        .send_raw(message, &recipients, Default::default())
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?;
    Ok(status_response(result.message_id))
}

#[utoipa::path(
    post,
    security(("bearer_auth" = [])),
    path = "/api/v1/sessions/{session_id}/status/video",
    tag = "status",
    params(("session_id" = String, Path, description = "Session ID")),
    request_body = StatusMediaRequest,
    responses(
        (status = 200, description = "Status posted", body = MessageResponse),
        (status = 400, description = "Invalid request"),
        (status = 404, description = "Session not found"),
        (status = 503, description = "Not connected")
    )
)]
pub async fn post_status_video(
    State(state): State<AppState>,
    Path(session_id): Path<String>,
    Json(request): Json<StatusMediaRequest>,
) -> Result<Json<MessageResponse>, ApiError> {
    let client = get_client(&state, &session_id)?;
    let recipients = parse_recipients(&request.recipients)?;
    let media = crate::handlers::messages::resolve_media(
        &client,
        &request.media,
        wacore::download::MediaType::Video,
    )
    .await?;
    let message = wa::Message {
        video_message: MessageField::some(wa::message::VideoMessage {
            url: Some(media.url),
            direct_path: Some(media.direct_path),
            media_key: Some(media.media_key),
            file_sha256: Some(media.file_sha256),
            file_enc_sha256: Some(media.file_enc_sha256),
            file_length: Some(media.file_length),
            mimetype: Some(media.mimetype),
            caption: request.caption,
            ..Default::default()
        }),
        ..Default::default()
    };
    let result = client
        .status()
        .send_raw(message, &recipients, Default::default())
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?;
    Ok(status_response(result.message_id))
}
