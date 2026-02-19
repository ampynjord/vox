use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use validator::Validate;

/// Liste exhaustive des permissions Vox
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum Permission {
    // Espace
    ManageSpace,         // Modifier nom/description/icône de l'espace
    ManageRoles,         // Créer/modifier/supprimer des rôles
    ManageChannels,      // Créer/modifier/supprimer des canaux
    KickMembers,         // Expulser des membres
    BanMembers,          // Bannir des membres
    InviteMembers,       // Créer des invitations
    // Canaux textuels
    ViewChannel,         // Voir le canal
    SendMessages,        // Envoyer des messages
    ManageMessages,      // Supprimer les messages d'autres personnes
    AttachFiles,         // Envoyer des fichiers
    EmbedLinks,          // Aperçu des liens
    AddReactions,        // Ajouter des réactions
    MentionEveryone,     // Mentionner @everyone
    // Canaux vocaux
    VoiceConnect,        // Se connecter au canal vocal
    VoiceSpeak,          // Parler dans le canal vocal
    VoiceMuteMembers,    // Rendre muet d'autres membres
    VoiceDeafenMembers,  // Rendre sourd d'autres membres
    VoiceMoveMembers,    // Déplacer des membres entre canaux
    ScreenShare,         // Partager son écran
}

impl Permission {
    pub fn as_str(&self) -> &'static str {
        match self {
            Permission::ManageSpace => "manage_space",
            Permission::ManageRoles => "manage_roles",
            Permission::ManageChannels => "manage_channels",
            Permission::KickMembers => "kick_members",
            Permission::BanMembers => "ban_members",
            Permission::InviteMembers => "invite_members",
            Permission::ViewChannel => "view_channel",
            Permission::SendMessages => "send_messages",
            Permission::ManageMessages => "manage_messages",
            Permission::AttachFiles => "attach_files",
            Permission::EmbedLinks => "embed_links",
            Permission::AddReactions => "add_reactions",
            Permission::MentionEveryone => "mention_everyone",
            Permission::VoiceConnect => "voice_connect",
            Permission::VoiceSpeak => "voice_speak",
            Permission::VoiceMuteMembers => "voice_mute_members",
            Permission::VoiceDeafenMembers => "voice_deafen_members",
            Permission::VoiceMoveMembers => "voice_move_members",
            Permission::ScreenShare => "screen_share",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "manage_space" => Some(Permission::ManageSpace),
            "manage_roles" => Some(Permission::ManageRoles),
            "manage_channels" => Some(Permission::ManageChannels),
            "kick_members" => Some(Permission::KickMembers),
            "ban_members" => Some(Permission::BanMembers),
            "invite_members" => Some(Permission::InviteMembers),
            "view_channel" => Some(Permission::ViewChannel),
            "send_messages" => Some(Permission::SendMessages),
            "manage_messages" => Some(Permission::ManageMessages),
            "attach_files" => Some(Permission::AttachFiles),
            "embed_links" => Some(Permission::EmbedLinks),
            "add_reactions" => Some(Permission::AddReactions),
            "mention_everyone" => Some(Permission::MentionEveryone),
            "voice_connect" => Some(Permission::VoiceConnect),
            "voice_speak" => Some(Permission::VoiceSpeak),
            "voice_mute_members" => Some(Permission::VoiceMuteMembers),
            "voice_deafen_members" => Some(Permission::VoiceDeafenMembers),
            "voice_move_members" => Some(Permission::VoiceMoveMembers),
            "screen_share" => Some(Permission::ScreenShare),
            _ => None,
        }
    }

    /// Permissions par défaut pour le rôle "member"
    pub fn default_member_permissions() -> Vec<Permission> {
        vec![
            Permission::ViewChannel,
            Permission::SendMessages,
            Permission::AttachFiles,
            Permission::EmbedLinks,
            Permission::AddReactions,
            Permission::VoiceConnect,
            Permission::VoiceSpeak,
            Permission::ScreenShare,
            Permission::InviteMembers,
        ]
    }

    /// Toutes les permissions (pour owner)
    pub fn all() -> Vec<Permission> {
        vec![
            Permission::ManageSpace,
            Permission::ManageRoles,
            Permission::ManageChannels,
            Permission::KickMembers,
            Permission::BanMembers,
            Permission::InviteMembers,
            Permission::ViewChannel,
            Permission::SendMessages,
            Permission::ManageMessages,
            Permission::AttachFiles,
            Permission::EmbedLinks,
            Permission::AddReactions,
            Permission::MentionEveryone,
            Permission::VoiceConnect,
            Permission::VoiceSpeak,
            Permission::VoiceMuteMembers,
            Permission::VoiceDeafenMembers,
            Permission::VoiceMoveMembers,
            Permission::ScreenShare,
        ]
    }
}

/// Rôle personnalisé au sein d'un espace
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Role {
    pub id: Uuid,
    pub space_id: Uuid,
    pub name: String,
    pub color: Option<String>,
    pub icon: Option<String>,
    pub position: i32,
    pub is_default: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Rôle avec ses permissions incluses
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoleWithPermissions {
    pub id: Uuid,
    pub space_id: Uuid,
    pub name: String,
    pub color: Option<String>,
    pub icon: Option<String>,
    pub position: i32,
    pub is_default: bool,
    pub permissions: Vec<String>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateRoleRequest {
    #[validate(length(min = 1, max = 64))]
    pub name: String,
    pub color: Option<String>,
    pub icon: Option<String>,
    pub permissions: Vec<String>,
    pub is_default: Option<bool>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateRoleRequest {
    #[validate(length(min = 1, max = 64))]
    pub name: Option<String>,
    pub color: Option<String>,
    pub icon: Option<String>,
    pub position: Option<i32>,
    pub permissions: Option<Vec<String>>,
}

/// Override de permission sur un canal
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ChannelPermissionOverride {
    pub channel_id: Uuid,
    pub role_id: Uuid,
    pub permission: String,
    pub allow: bool,
}

/// Attribution d'un rôle à un membre
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct MemberRole {
    pub space_id: Uuid,
    pub user_id: Uuid,
    pub role_id: Uuid,
    pub assigned_at: DateTime<Utc>,
}
