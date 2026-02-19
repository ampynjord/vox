# 🎙️ Vox

**Réseau social vocal/textuel respectueux de la vie privée**

Vox est une plateforme de communication temps réel axée sur la performance, la transparence et le respect des données personnelles.

## Architecture

```
vox/
├── backend/              # Backend Rust (Axum)
│   ├── api/              # API REST (auth, CRUD)
│   ├── gateway/          # WebSocket gateway (temps réel)
│   ├── shared/           # Types partagés, auth, validation
│   ├── voice/            # Serveur WebRTC (à venir)
│   ├── media/            # Upload/traitement fichiers (à venir)
│   └── migrations/       # Migrations PostgreSQL
├── frontend/
│   └── web/              # App React + Vite + TypeScript
├── infra/                # Dockerfiles
├── protocol/             # Spécification du protocole
└── docker-compose.yml    # Stack complète
```

## Stack technique

- **Backend**: Rust + Axum + SQLx + Tokio
- **Base de données**: PostgreSQL 16
- **Cache/Présence**: Redis 7
- **Frontend**: React 19 + TypeScript + Vite
- **State management**: Zustand
- **Temps réel**: WebSocket natif

## Démarrage rapide

### Prérequis

- Docker + Docker Compose
- Rust 1.75+ (pour le dev local)
- Node.js 20+ (pour le frontend)

### Avec Docker

```bash
# Démarrer toute la stack
docker compose up -d

# L'API sera disponible sur http://localhost:8080
# Le gateway WebSocket sur ws://localhost:8081
```

### En développement local

```bash
# 1. Démarrer les services
docker compose up postgres redis -d

# 2. Backend API
cd backend
cp .env.example .env  # Ajuster si nécessaire
cargo run -p vox-api

# 3. Backend Gateway (dans un autre terminal)
cargo run -p vox-gateway

# 4. Frontend
cd frontend/web
npm install
npm run dev
```

## API Endpoints

### Auth
- `POST /api/v1/auth/register` - Inscription
- `POST /api/v1/auth/login` - Connexion

### Espaces
- `GET /api/v1/spaces` - Mes espaces
- `POST /api/v1/spaces` - Créer un espace
- `GET /api/v1/spaces/:id` - Détails d'un espace
- `PUT /api/v1/spaces/:id` - Modifier un espace
- `DELETE /api/v1/spaces/:id` - Supprimer un espace
- `POST /api/v1/spaces/:id/join` - Rejoindre
- `POST /api/v1/spaces/:id/leave` - Quitter

### Canaux
- `GET /api/v1/channels/space/:space_id` - Canaux d'un espace
- `POST /api/v1/channels/space/:space_id` - Créer un canal
- `PUT /api/v1/channels/:id` - Modifier un canal
- `DELETE /api/v1/channels/:id` - Supprimer un canal

### Messages
- `GET /api/v1/messages/channel/:channel_id` - Messages d'un canal
- `POST /api/v1/messages/channel/:channel_id` - Envoyer un message
- `PUT /api/v1/messages/:id` - Modifier un message
- `DELETE /api/v1/messages/:id` - Supprimer un message

### WebSocket Gateway
- `ws://localhost:8081/ws?token=<jwt>` - Connexion temps réel

## Licence

Proprietary code, all rights reserved (see [LICENSE](LICENSE)).
