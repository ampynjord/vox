use std::sync::Arc;
use tokio::sync::broadcast;
use uuid::Uuid;
use chrono::Utc;

use crate::events::{ClientEvent, ServerEvent};
use crate::hub::Hub;

/// Gère la logique d'une session WebSocket individuelle
pub struct Session {
    pub session_id: String,
    pub user_id: Uuid,
    pub username: String,
    hub: Arc<Hub>,
}

impl Session {
    pub fn new(session_id: String, user_id: Uuid, username: String, hub: Arc<Hub>) -> Self {
        Self {
            session_id,
            user_id,
            username,
            hub,
        }
    }

    /// Traite un événement reçu du client
    pub async fn handle_event(&self, event: ClientEvent) -> Option<ServerEvent> {
        match event {
            ClientEvent::Subscribe { channel_id } => {
                self.hub.subscribe_channel(&self.session_id, channel_id).await;
                tracing::debug!("{} s'abonne au canal {}", self.username, channel_id);
                None
            }
            ClientEvent::Unsubscribe { channel_id } => {
                self.hub.unsubscribe_channel(&self.session_id, channel_id).await;
                tracing::debug!("{} se désabonne du canal {}", self.username, channel_id);
                None
            }
            ClientEvent::SendMessage {
                channel_id,
                content,
                reply_to,
            } => {
                let msg_id = Uuid::new_v4();
                let now = Utc::now();

                let event = ServerEvent::NewMessage {
                    id: msg_id,
                    channel_id,
                    author_id: self.user_id,
                    author_username: self.username.clone(),
                    author_display_name: self.username.clone(), // TODO: récupérer le display_name
                    author_avatar_url: None,
                    content,
                    reply_to,
                    created_at: now,
                };

                // Broadcast à tous les abonnés du canal
                self.hub.broadcast_to_channel(channel_id, event).await;

                // TODO: Persister le message via un appel interne à l'API
                // ou directement en DB

                None
            }
            ClientEvent::StartTyping { channel_id } => {
                let event = ServerEvent::UserTyping {
                    channel_id,
                    user_id: self.user_id,
                    username: self.username.clone(),
                };
                self.hub.broadcast_to_channel(channel_id, event).await;
                None
            }
            ClientEvent::UpdateStatus { status } => {
                let event = ServerEvent::PresenceUpdate {
                    user_id: self.user_id,
                    status,
                };
                // TODO: broadcast aux espaces communs
                tracing::debug!("{} a changé de statut", self.username);
                Some(event)
            }
            ClientEvent::Ping => Some(ServerEvent::Pong),
            ClientEvent::JoinVoice { channel_id } => {
                let participants = self.hub
                    .join_voice(self.user_id, &self.username, channel_id)
                    .await;
                tracing::info!(
                    "{} a rejoint le vocal {} ({} participants existants)",
                    self.username,
                    channel_id,
                    participants.len()
                );
                // On renvoie la liste des participants pour que le client initie les connexions WebRTC
                Some(ServerEvent::VoiceUserJoined {
                    channel_id,
                    user_id: self.user_id,
                    username: self.username.clone(),
                })
            }
            ClientEvent::LeaveVoice { channel_id } => {
                self.hub.leave_voice(self.user_id, channel_id).await;
                None
            }
            ClientEvent::UpdateVoiceState {
                is_muted,
                is_deafened,
                is_screen_sharing,
            } => {
                if let Some(channel_id) = self.hub.get_user_voice_channel(self.user_id).await {
                    self.hub.broadcast_voice_state(
                        channel_id,
                        self.user_id,
                        is_muted.unwrap_or(false),
                        is_deafened.unwrap_or(false),
                        is_screen_sharing.unwrap_or(false),
                    ).await;
                }
                None
            }
            ClientEvent::VoiceSignal {
                target_user_id,
                signal_type,
                sdp,
                candidate,
                sdp_mid,
                sdp_m_line_index,
            } => {
                self.hub.relay_voice_signal(
                    self.user_id,
                    target_user_id,
                    signal_type,
                    sdp,
                    candidate,
                    sdp_mid,
                    sdp_m_line_index,
                ).await;
                None
            }
        }
    }
}
