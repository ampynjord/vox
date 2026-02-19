use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// Session vocale active d'un utilisateur dans un canal
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct VoiceSession {
    pub id: Uuid,
    pub channel_id: Uuid,
    pub user_id: Uuid,
    pub is_muted: bool,
    pub is_deafened: bool,
    pub is_screen_sharing: bool,
    pub joined_at: DateTime<Utc>,
}

/// Vue enrichie d'un participant vocal (avec infos user)
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct VoiceParticipant {
    pub id: Uuid,
    pub channel_id: Uuid,
    pub user_id: Uuid,
    pub username: String,
    pub display_name: String,
    pub avatar_url: Option<String>,
    pub is_muted: bool,
    pub is_deafened: bool,
    pub is_screen_sharing: bool,
    pub joined_at: DateTime<Utc>,
}

/// Requête pour rejoindre ou quitter un canal vocal
#[derive(Debug, Deserialize)]
pub struct JoinVoiceRequest {
    pub channel_id: Uuid,
}

/// Requête pour mettre à jour son état vocal
#[derive(Debug, Deserialize)]
pub struct UpdateVoiceStateRequest {
    pub is_muted: Option<bool>,
    pub is_deafened: Option<bool>,
    pub is_screen_sharing: Option<bool>,
}

/// Signaling WebRTC — offre/réponse/ICE candidate
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "signal_type")]
pub enum VoiceSignal {
    /// Offre SDP
    Offer {
        sdp: String,
        target_user_id: Uuid,
    },
    /// Réponse SDP
    Answer {
        sdp: String,
        target_user_id: Uuid,
    },
    /// Candidat ICE
    IceCandidate {
        candidate: String,
        sdp_mid: Option<String>,
        sdp_m_line_index: Option<u32>,
        target_user_id: Uuid,
    },
}
