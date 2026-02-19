pub mod auth;
pub mod spaces;
pub mod channels;
pub mod messages;
pub mod files;
pub mod roles;
pub mod voice;

use axum::Router;
use crate::state::AppState;

pub fn auth_routes() -> Router<AppState> {
    auth::router()
}

pub fn space_routes() -> Router<AppState> {
    spaces::router()
}

pub fn channel_routes() -> Router<AppState> {
    channels::router()
}

pub fn message_routes() -> Router<AppState> {
    messages::router()
}

pub fn file_routes() -> Router<AppState> {
    files::router()
}

pub fn role_routes() -> Router<AppState> {
    roles::router()
}

pub fn voice_routes() -> Router<AppState> {
    voice::router()
}
