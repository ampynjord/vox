use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};
use uuid::Uuid;
use redis::aio::ConnectionManager;
use redis::AsyncCommands;

use crate::events::ServerEvent;

/// Session connectée
#[derive(Debug, Clone)]
pub struct ConnectedUser {
    pub user_id: Uuid,
    pub username: String,
    pub session_id: String,
    pub sender: broadcast::Sender<ServerEvent>,
    pub subscribed_channels: HashSet<Uuid>,
}

/// Hub central qui gère toutes les connexions WebSocket
pub struct Hub {
    /// Map de session_id -> ConnectedUser
    connections: RwLock<HashMap<String, ConnectedUser>>,
    /// Map de user_id -> Vec<session_id> (un utilisateur peut avoir plusieurs sessions)
    user_sessions: RwLock<HashMap<Uuid, Vec<String>>>,
    /// Map de channel_id -> Set<session_id> (abonnements aux canaux)
    channel_subscribers: RwLock<HashMap<Uuid, HashSet<String>>>,
    /// Map de channel_id -> Set<user_id> (participants vocaux)
    voice_participants: RwLock<HashMap<Uuid, HashSet<Uuid>>>,
    /// Map de user_id -> channel_id (canal vocal actuel de chaque user)
    user_voice_channel: RwLock<HashMap<Uuid, Uuid>>,
    /// Connexion Redis pour la présence
    redis: ConnectionManager,
}

impl Hub {
    pub fn new(redis: ConnectionManager) -> Self {
        Self {
            connections: RwLock::new(HashMap::new()),
            user_sessions: RwLock::new(HashMap::new()),
            channel_subscribers: RwLock::new(HashMap::new()),
            voice_participants: RwLock::new(HashMap::new()),
            user_voice_channel: RwLock::new(HashMap::new()),
            redis,
        }
    }

    /// Enregistrer une nouvelle connexion
    pub async fn register(
        &self,
        session_id: String,
        user_id: Uuid,
        username: String,
        sender: broadcast::Sender<ServerEvent>,
    ) {
        let user = ConnectedUser {
            user_id,
            username: username.clone(),
            session_id: session_id.clone(),
            sender,
            subscribed_channels: HashSet::new(),
        };

        self.connections.write().await.insert(session_id.clone(), user);
        
        self.user_sessions
            .write()
            .await
            .entry(user_id)
            .or_default()
            .push(session_id);

        // Mettre à jour la présence Redis
        let mut redis = self.redis.clone();
        let _: Result<(), _> = redis.set_ex(
            format!("presence:{}", user_id),
            "online",
            3600, // 1h TTL
        ).await;

        tracing::info!("Utilisateur {} ({}) connecté", username, user_id);
    }

    /// Retirer une connexion
    pub async fn unregister(&self, session_id: &str) {
        let user = self.connections.write().await.remove(session_id);
        
        if let Some(user) = user {
            // Retirer de tous les canaux
            let mut channel_subs = self.channel_subscribers.write().await;
            for channel_id in &user.subscribed_channels {
                if let Some(subs) = channel_subs.get_mut(channel_id) {
                    subs.remove(session_id);
                }
            }
            drop(channel_subs);

            // Retirer de user_sessions
            let mut sessions = self.user_sessions.write().await;
            if let Some(user_sessions) = sessions.get_mut(&user.user_id) {
                user_sessions.retain(|s| s != session_id);
                if user_sessions.is_empty() {
                    sessions.remove(&user.user_id);
                    
                    // Plus aucune session → quitter le vocal et mettre hors ligne
                    if let Some(voice_channel) = self.user_voice_channel.read().await.get(&user.user_id).copied() {
                        drop(sessions);
                        self.leave_voice(user.user_id, voice_channel).await;
                    } else {
                        drop(sessions);
                    }

                    let mut redis = self.redis.clone();
                    let _: Result<(), _> = redis.del(
                        format!("presence:{}", user.user_id)
                    ).await;
                }
            }

            tracing::info!("Utilisateur {} ({}) déconnecté", user.username, user.user_id);
        }
    }

    /// Abonner une session à un canal
    pub async fn subscribe_channel(&self, session_id: &str, channel_id: Uuid) {
        self.channel_subscribers
            .write()
            .await
            .entry(channel_id)
            .or_default()
            .insert(session_id.to_string());

        if let Some(conn) = self.connections.write().await.get_mut(session_id) {
            conn.subscribed_channels.insert(channel_id);
        }
    }

    /// Désabonner une session d'un canal
    pub async fn unsubscribe_channel(&self, session_id: &str, channel_id: Uuid) {
        if let Some(subs) = self.channel_subscribers.write().await.get_mut(&channel_id) {
            subs.remove(session_id);
        }

        if let Some(conn) = self.connections.write().await.get_mut(session_id) {
            conn.subscribed_channels.remove(&channel_id);
        }
    }

    /// Broadcast un événement à tous les abonnés d'un canal
    pub async fn broadcast_to_channel(&self, channel_id: Uuid, event: ServerEvent) {
        let channel_subs = self.channel_subscribers.read().await;
        let connections = self.connections.read().await;

        if let Some(subscribers) = channel_subs.get(&channel_id) {
            for session_id in subscribers {
                if let Some(conn) = connections.get(session_id) {
                    let _ = conn.sender.send(event.clone());
                }
            }
        }
    }

    /// Envoyer un événement à un utilisateur spécifique (toutes ses sessions)
    pub async fn send_to_user(&self, user_id: Uuid, event: ServerEvent) {
        let sessions = self.user_sessions.read().await;
        let connections = self.connections.read().await;

        if let Some(user_sessions) = sessions.get(&user_id) {
            for session_id in user_sessions {
                if let Some(conn) = connections.get(session_id) {
                    let _ = conn.sender.send(event.clone());
                }
            }
        }
    }

    /// Nombre de connexions actives
    pub async fn connection_count(&self) -> usize {
        self.connections.read().await.len()
    }

    // ========== Voice Methods ==========

    /// Un utilisateur rejoint un canal vocal
    pub async fn join_voice(&self, user_id: Uuid, username: &str, channel_id: Uuid) -> Vec<Uuid> {
        // D'abord quitter l'ancien canal vocal s'il y en a un
        if let Some(old_channel) = self.user_voice_channel.read().await.get(&user_id).copied() {
            if old_channel != channel_id {
                self.leave_voice(user_id, old_channel).await;
            }
        }

        // Ajouter à la map des participants
        self.voice_participants
            .write()
            .await
            .entry(channel_id)
            .or_default()
            .insert(user_id);

        self.user_voice_channel
            .write()
            .await
            .insert(user_id, channel_id);

        // Stocker dans Redis
        let mut redis = self.redis.clone();
        let _: Result<(), _> = redis.set_ex(
            format!("voice:{}:{}", channel_id, user_id),
            "connected",
            7200,
        ).await;

        // Notifier tous les participants existants
        let event = ServerEvent::VoiceUserJoined {
            channel_id,
            user_id,
            username: username.to_string(),
        };
        self.broadcast_to_channel(channel_id, event).await;

        tracing::info!("{} a rejoint le canal vocal {}", username, channel_id);

        // Retourner la liste des autres participants pour le peer-to-peer
        let participants = self.voice_participants.read().await;
        participants
            .get(&channel_id)
            .map(|p| p.iter().filter(|&&id| id != user_id).copied().collect())
            .unwrap_or_default()
    }

    /// Un utilisateur quitte un canal vocal
    pub async fn leave_voice(&self, user_id: Uuid, channel_id: Uuid) {
        // Retirer de la map des participants
        let mut participants = self.voice_participants.write().await;
        if let Some(channel_users) = participants.get_mut(&channel_id) {
            channel_users.remove(&user_id);
            if channel_users.is_empty() {
                participants.remove(&channel_id);
            }
        }
        drop(participants);

        self.user_voice_channel.write().await.remove(&user_id);

        // Retirer de Redis
        let mut redis = self.redis.clone();
        let _: Result<(), _> = redis.del(
            format!("voice:{}:{}", channel_id, user_id)
        ).await;

        // Notifier les autres participants
        let event = ServerEvent::VoiceUserLeft {
            channel_id,
            user_id,
        };
        self.broadcast_to_channel(channel_id, event).await;

        tracing::info!("User {} a quitté le canal vocal {}", user_id, channel_id);
    }

    /// Relayer un signal WebRTC d'un utilisateur à un autre
    pub async fn relay_voice_signal(
        &self,
        from_user_id: Uuid,
        target_user_id: Uuid,
        signal_type: String,
        sdp: Option<String>,
        candidate: Option<String>,
        sdp_mid: Option<String>,
        sdp_m_line_index: Option<u32>,
    ) {
        let event = ServerEvent::VoiceSignal {
            from_user_id,
            signal_type,
            sdp,
            candidate,
            sdp_mid,
            sdp_m_line_index,
        };
        self.send_to_user(target_user_id, event).await;
    }

    /// Broadcast une mise à jour d'état vocal
    pub async fn broadcast_voice_state(
        &self,
        channel_id: Uuid,
        user_id: Uuid,
        is_muted: bool,
        is_deafened: bool,
        is_screen_sharing: bool,
    ) {
        let event = ServerEvent::VoiceStateUpdate {
            channel_id,
            user_id,
            is_muted,
            is_deafened,
            is_screen_sharing,
        };
        self.broadcast_to_channel(channel_id, event).await;
    }

    /// Récupérer les participants d'un canal vocal
    pub async fn get_voice_participants(&self, channel_id: Uuid) -> Vec<Uuid> {
        self.voice_participants
            .read()
            .await
            .get(&channel_id)
            .map(|p| p.iter().copied().collect())
            .unwrap_or_default()
    }

    /// Récupérer le canal vocal d'un utilisateur
    pub async fn get_user_voice_channel(&self, user_id: Uuid) -> Option<Uuid> {
        self.user_voice_channel.read().await.get(&user_id).copied()
    }
}
