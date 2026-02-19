use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

/// Événements envoyés par le client
#[derive(Debug, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum ClientEvent {
    /// S'abonner à un canal pour recevoir les messages
    Subscribe { channel_id: Uuid },
    /// Se désabonner d'un canal
    Unsubscribe { channel_id: Uuid },
    /// Envoyer un message dans un canal
    SendMessage {
        channel_id: Uuid,
        content: String,
        reply_to: Option<Uuid>,
    },
    /// Indicateur de frappe
    StartTyping { channel_id: Uuid },
    /// Mettre à jour son statut
    UpdateStatus { status: String },
    /// Ping pour garder la connexion alive
    Ping,
    /// Rejoindre un canal vocal
    JoinVoice { channel_id: Uuid },
    /// Quitter le canal vocal
    LeaveVoice { channel_id: Uuid },
    /// Mettre à jour son état vocal (mute/deaf/screenshare)
    UpdateVoiceState {
        is_muted: Option<bool>,
        is_deafened: Option<bool>,
        is_screen_sharing: Option<bool>,
    },
    /// Signaling WebRTC (SDP offer/answer, ICE candidates)
    VoiceSignal {
        target_user_id: Uuid,
        signal_type: String,
        sdp: Option<String>,
        candidate: Option<String>,
        sdp_mid: Option<String>,
        sdp_m_line_index: Option<u32>,
    },
}

/// Événements envoyés par le serveur
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", content = "data")]
pub enum ServerEvent {
    /// Connexion réussie
    Ready {
        user_id: Uuid,
        session_id: String,
    },
    /// Nouveau message reçu
    NewMessage {
        id: Uuid,
        channel_id: Uuid,
        author_id: Uuid,
        author_username: String,
        author_display_name: String,
        author_avatar_url: Option<String>,
        content: String,
        reply_to: Option<Uuid>,
        created_at: DateTime<Utc>,
    },
    /// Message édité
    MessageEdited {
        id: Uuid,
        channel_id: Uuid,
        content: String,
        updated_at: DateTime<Utc>,
    },
    /// Message supprimé
    MessageDeleted {
        id: Uuid,
        channel_id: Uuid,
    },
    /// Quelqu'un est en train de taper
    UserTyping {
        channel_id: Uuid,
        user_id: Uuid,
        username: String,
    },
    /// Changement de présence d'un utilisateur
    PresenceUpdate {
        user_id: Uuid,
        status: String,
    },
    /// Membre rejoint un espace
    MemberJoined {
        space_id: Uuid,
        user_id: Uuid,
        username: String,
    },
    /// Membre quitte un espace
    MemberLeft {
        space_id: Uuid,
        user_id: Uuid,
    },
    /// Pong en réponse au ping
    Pong,
    /// Un utilisateur a rejoint le canal vocal
    VoiceUserJoined {
        channel_id: Uuid,
        user_id: Uuid,
        username: String,
    },
    /// Un utilisateur a quitté le canal vocal
    VoiceUserLeft {
        channel_id: Uuid,
        user_id: Uuid,
    },
    /// État vocal mis à jour (mute/deaf/screenshare)
    VoiceStateUpdate {
        channel_id: Uuid,
        user_id: Uuid,
        is_muted: bool,
        is_deafened: bool,
        is_screen_sharing: bool,
    },
    /// Signal WebRTC reçu (relayé d'un pair à l'autre)
    VoiceSignal {
        from_user_id: Uuid,
        signal_type: String,
        sdp: Option<String>,
        candidate: Option<String>,
        sdp_mid: Option<String>,
        sdp_m_line_index: Option<u32>,
    },
    /// Erreur
    Error {
        message: String,
    },
}
