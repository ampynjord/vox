# 🎙️ Vox

**Privacy-respecting voice and text social network**

Vox is a real-time communication platform focused on performance, transparency and respect for personal data.

## Architecture

```
vox/
├── backend/              # Rust backend (Axum)
│   ├── api/              # REST API (auth, CRUD)
│   ├── gateway/          # WebSocket gateway (real time)
│   ├── shared/           # Shared types, auth, validation
│   ├── voice/            # WebRTC server (coming soon)
│   ├── media/            # File upload/processing (coming soon)
│   └── migrations/       # PostgreSQL migrations
├── frontend/
│   └── web/              # React + Vite + TypeScript app
├── infra/                # Dockerfiles
├── protocol/             # Protocol specification
└── docker-compose.yml    # Full stack
```

## Tech stack

- **Backend**: Rust + Axum + SQLx + Tokio
- **Database**: PostgreSQL 16
- **Cache/Presence**: Redis 7
- **Frontend**: React 19 + TypeScript + Vite
- **State management**: Zustand
- **Real time**: native WebSocket

## Quick start

### Requirements

- Docker + Docker Compose
- Rust 1.75+ (for local development)
- Node.js 20+ (for the frontend)

### With Docker

```bash
# Start the whole stack
docker compose up -d

# The API will be available at http://localhost:8080
# The WebSocket gateway at ws://localhost:8081
```

### Local development

```bash
# 1. Start the services
docker compose up postgres redis -d

# 2. Backend API
cd backend
cp .env.example .env  # Adjust if needed
cargo run -p vox-api

# 3. Backend Gateway (in another terminal)
cargo run -p vox-gateway

# 4. Frontend
cd frontend/web
npm install
npm run dev
```

## API endpoints

### Auth
- `POST /api/v1/auth/register` - Sign up
- `POST /api/v1/auth/login` - Log in

### Spaces
- `GET /api/v1/spaces` - My spaces
- `POST /api/v1/spaces` - Create a space
- `GET /api/v1/spaces/:id` - Space details
- `PUT /api/v1/spaces/:id` - Update a space
- `DELETE /api/v1/spaces/:id` - Delete a space
- `POST /api/v1/spaces/:id/join` - Join
- `POST /api/v1/spaces/:id/leave` - Leave

### Channels
- `GET /api/v1/channels/space/:space_id` - Channels of a space
- `POST /api/v1/channels/space/:space_id` - Create a channel
- `PUT /api/v1/channels/:id` - Update a channel
- `DELETE /api/v1/channels/:id` - Delete a channel

### Messages
- `GET /api/v1/messages/channel/:channel_id` - Messages of a channel
- `POST /api/v1/messages/channel/:channel_id` - Send a message
- `PUT /api/v1/messages/:id` - Edit a message
- `DELETE /api/v1/messages/:id` - Delete a message

### WebSocket Gateway
- `ws://localhost:8081/ws?token=<jwt>` - Real-time connection

## License

Proprietary code, all rights reserved (see [LICENSE](LICENSE)).