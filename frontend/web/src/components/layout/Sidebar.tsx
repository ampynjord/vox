import React, { useEffect, useState } from 'react';
import { useAppStore } from '../../stores/appStore';
import { Hash, Volume2, Megaphone, Plus, ChevronDown, LogOut } from 'lucide-react';
import { VoicePanel } from '../voice/VoicePanel';
import { CreateSpaceModal } from '../modals/CreateSpaceModal';
import { CreateChannelModal } from '../modals/CreateChannelModal';
import { useAuthStore } from '../../stores/authStore';
import './Layout.css';

export const Sidebar: React.FC = () => {
  const {
    spaces,
    currentSpace,
    channels,
    currentChannel,
    loadSpaces,
    selectSpace,
    selectChannel,
  } = useAppStore();
  const { user, logout } = useAuthStore();
  const [showCreateSpace, setShowCreateSpace] = useState(false);
  const [showCreateChannel, setShowCreateChannel] = useState(false);

  useEffect(() => {
    loadSpaces();
  }, [loadSpaces]);

  const getChannelIcon = (type: string) => {
    switch (type.toLowerCase()) {
      case 'voice': return <Volume2 size={16} />;
      case 'announcement': return <Megaphone size={16} />;
      default: return <Hash size={16} />;
    }
  };

  return (
    <div className="sidebar">
      {/* Barre des espaces */}
      <div className="space-bar">
        {spaces.map((space) => (
          <button
            key={space.id}
            className={`space-icon ${currentSpace?.id === space.id ? 'active' : ''}`}
            onClick={() => selectSpace(space)}
            title={space.name}
          >
            {space.icon_url ? (
              <img src={space.icon_url} alt={space.name} />
            ) : (
              <span>{space.name.charAt(0).toUpperCase()}</span>
            )}
          </button>
        ))}
        <button
          className="space-icon add-space"
          title="Créer un espace"
          onClick={() => setShowCreateSpace(true)}
        >
          <Plus size={20} />
        </button>
      </div>

      {/* Liste des canaux de l'espace sélectionné */}
      <div className="channel-list">
        {currentSpace ? (
          <>
            <div className="channel-header">
              <h2>{currentSpace.name}</h2>
              <ChevronDown size={16} />
            </div>

            <div className="channel-category">
              <div className="category-header">
                <span className="category-label">Canaux textuels</span>
                <button
                  className="add-channel-btn"
                  title="Créer un canal"
                  onClick={() => setShowCreateChannel(true)}
                >
                  <Plus size={14} />
                </button>
              </div>
              {channels
                .filter((c) => c.channel_type.toLowerCase() === 'text')
                .map((channel) => (
                  <button
                    key={channel.id}
                    className={`channel-item ${currentChannel?.id === channel.id ? 'active' : ''}`}
                    onClick={() => selectChannel(channel)}
                  >
                    {getChannelIcon(channel.channel_type)}
                    <span>{channel.name}</span>
                    {channel.is_ephemeral && <span className="ephemeral-badge">Éphémère</span>}
                  </button>
                ))}
            </div>

            <div className="channel-category">
              <span className="category-label">Canaux vocaux</span>
              {channels
                .filter((c) => c.channel_type.toLowerCase() === 'voice')
                .map((channel) => (
                  <div key={channel.id}>
                    <button
                      className={`channel-item ${currentChannel?.id === channel.id ? 'active' : ''}`}
                      onClick={() => selectChannel(channel)}
                    >
                      {getChannelIcon(channel.channel_type)}
                      <span>{channel.name}</span>
                    </button>
                    {currentChannel?.id === channel.id && (
                      <VoicePanel channelId={channel.id} channelName={channel.name} />
                    )}
                  </div>
                ))}
            </div>
          </>
        ) : (
          <div className="channel-list-empty">
            <p>Créez ou rejoignez un espace pour commencer</p>
          </div>
        )}

        {/* Barre utilisateur */}
        <div className="user-bar">
          <div className="user-info">
            <div className="user-avatar">
              {user?.display_name?.charAt(0).toUpperCase() || '?'}
            </div>
            <div className="user-details">
              <span className="user-display-name">{user?.display_name}</span>
              <span className="user-username">@{user?.username}</span>
            </div>
          </div>
          <button className="logout-btn" onClick={logout} title="Se déconnecter">
            <LogOut size={16} />
          </button>
        </div>
      </div>

      {/* Modals */}
      <CreateSpaceModal
        isOpen={showCreateSpace}
        onClose={() => setShowCreateSpace(false)}
      />
      {currentSpace && (
        <CreateChannelModal
          isOpen={showCreateChannel}
          onClose={() => setShowCreateChannel(false)}
          spaceId={currentSpace.id}
        />
      )}
    </div>
  );
};
