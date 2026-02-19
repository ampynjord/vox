-- =============================================
-- Phase 2 Migration: Voix, Fichiers, Permissions
-- =============================================

-- =============================================
-- Table des sessions vocales (qui est dans quel canal vocal)
-- =============================================
CREATE TABLE voice_sessions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    channel_id UUID NOT NULL REFERENCES channels(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    is_muted BOOLEAN NOT NULL DEFAULT false,
    is_deafened BOOLEAN NOT NULL DEFAULT false,
    is_screen_sharing BOOLEAN NOT NULL DEFAULT false,
    joined_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (channel_id, user_id)
);

CREATE INDEX idx_voice_sessions_channel ON voice_sessions(channel_id);
CREATE INDEX idx_voice_sessions_user ON voice_sessions(user_id);

-- =============================================
-- Table des fichiers (pièces jointes)
-- =============================================
CREATE TYPE file_status AS ENUM ('uploading', 'ready', 'failed', 'deleted');

CREATE TABLE files (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    uploader_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    message_id UUID REFERENCES messages(id) ON DELETE SET NULL,
    channel_id UUID NOT NULL REFERENCES channels(id) ON DELETE CASCADE,
    filename VARCHAR(255) NOT NULL,
    original_filename VARCHAR(255) NOT NULL,
    mime_type VARCHAR(127) NOT NULL,
    size_bytes BIGINT NOT NULL,
    status file_status NOT NULL DEFAULT 'uploading',
    url TEXT,
    thumbnail_url TEXT,
    checksum VARCHAR(64),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_files_message ON files(message_id);
CREATE INDEX idx_files_channel ON files(channel_id);
CREATE INDEX idx_files_uploader ON files(uploader_id);

-- =============================================
-- Système de permissions granulaires
-- =============================================

-- Table des rôles personnalisés par espace
CREATE TABLE roles (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    space_id UUID NOT NULL REFERENCES spaces(id) ON DELETE CASCADE,
    name VARCHAR(64) NOT NULL,
    color VARCHAR(7),  -- Couleur hex (#FF5733)
    icon VARCHAR(64),  -- Emoji ou icône
    position INTEGER NOT NULL DEFAULT 0,  -- Ordre de priorité (plus haut = plus de pouvoir)
    is_default BOOLEAN NOT NULL DEFAULT false,  -- Rôle attribué aux nouveaux membres
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (space_id, name)
);

CREATE INDEX idx_roles_space ON roles(space_id);

-- Permissions possibles pour un rôle
CREATE TABLE role_permissions (
    role_id UUID NOT NULL REFERENCES roles(id) ON DELETE CASCADE,
    permission VARCHAR(64) NOT NULL,
    PRIMARY KEY (role_id, permission)
);

-- Attribution des rôles aux membres
CREATE TABLE member_roles (
    space_id UUID NOT NULL,
    user_id UUID NOT NULL,
    role_id UUID NOT NULL REFERENCES roles(id) ON DELETE CASCADE,
    assigned_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (space_id, user_id, role_id),
    FOREIGN KEY (space_id, user_id) REFERENCES space_members(space_id, user_id) ON DELETE CASCADE
);

CREATE INDEX idx_member_roles_role ON member_roles(role_id);

-- Overrides de permissions par canal (pour restreindre ou accorder per-channel)
CREATE TABLE channel_permission_overrides (
    channel_id UUID NOT NULL REFERENCES channels(id) ON DELETE CASCADE,
    role_id UUID NOT NULL REFERENCES roles(id) ON DELETE CASCADE,
    permission VARCHAR(64) NOT NULL,
    allow BOOLEAN NOT NULL,  -- true = accorder, false = refuser
    PRIMARY KEY (channel_id, role_id, permission)
);

-- Ajouter la colonne has_attachments aux messages
ALTER TABLE messages ADD COLUMN has_attachments BOOLEAN NOT NULL DEFAULT false;

-- Trigger updated_at pour roles
CREATE TRIGGER update_roles_updated_at BEFORE UPDATE ON roles
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
