use std::sync::Arc;
use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Query, State,
    },
    response::IntoResponse,
};
use futures_util::{SinkExt, StreamExt};
use serde::Deserialize;
use tokio::sync::broadcast;
use uuid::Uuid;

use vox_shared::auth::validate_token;

use crate::events::{ClientEvent, ServerEvent};
use crate::session::Session;
use crate::GatewayState;

#[derive(Deserialize)]
pub struct WsQuery {
    pub token: String,
}

/// Handler WebSocket — le client se connecte avec ?token=<jwt>
pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<GatewayState>>,
    Query(query): Query<WsQuery>,
) -> impl IntoResponse {
    // Valider le token avant d'accepter la connexion
    match validate_token(&query.token, &state.jwt_secret) {
        Ok(claims) => {
            ws.on_upgrade(move |socket| handle_socket(socket, state, claims.sub, claims.username))
        }
        Err(_) => {
            // Rejeter la connexion
            ws.on_upgrade(|mut socket| async move {
                let err = ServerEvent::Error {
                    message: "Token invalide".to_string(),
                };
                let _ = socket
                    .send(Message::Text(serde_json::to_string(&err).unwrap().into()))
                    .await;
                let _ = socket.close().await;
            })
        }
    }
}

async fn handle_socket(
    socket: WebSocket,
    state: Arc<GatewayState>,
    user_id: Uuid,
    username: String,
) {
    let (mut ws_sender, mut ws_receiver) = socket.split();
    let session_id = Uuid::new_v4().to_string();

    // Canal broadcast pour envoyer des événements à ce client
    let (tx, mut rx) = broadcast::channel::<ServerEvent>(256);

    // Enregistrer la session dans le hub
    state.hub.register(session_id.clone(), user_id, username.clone(), tx).await;

    // Envoyer l'événement Ready
    let ready = ServerEvent::Ready {
        user_id,
        session_id: session_id.clone(),
    };
    let _ = ws_sender
        .send(Message::Text(serde_json::to_string(&ready).unwrap().into()))
        .await;

    let session = Arc::new(Session::new(
        session_id.clone(),
        user_id,
        username,
        state.hub.clone(),
    ));

    // Task pour envoyer les événements broadcast au client
    let send_task = tokio::spawn(async move {
        while let Ok(event) = rx.recv().await {
            let msg = serde_json::to_string(&event).unwrap();
            if ws_sender.send(Message::Text(msg.into())).await.is_err() {
                break;
            }
        }
    });

    // Task pour recevoir les messages du client
    let recv_session = session.clone();
    let recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = ws_receiver.next().await {
            match msg {
                Message::Text(text) => {
                    match serde_json::from_str::<ClientEvent>(&text) {
                        Ok(event) => {
                            let _response = recv_session.handle_event(event).await;
                        }
                        Err(e) => {
                            tracing::warn!("Événement invalide: {}", e);
                        }
                    }
                }
                Message::Close(_) => break,
                _ => {}
            }
        }
    });

    // Attendre que l'une des tasks se termine
    tokio::select! {
        _ = send_task => {},
        _ = recv_task => {},
    }

    // Nettoyer
    state.hub.unregister(&session_id).await;
    tracing::info!("Session {} terminée", session_id);
}
