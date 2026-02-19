import React, { useRef, useEffect, useState } from 'react';
import { useAppStore } from '../../stores/appStore';
import { useAuthStore } from '../../stores/authStore';
import { Send, Reply, Edit3, Trash2 } from 'lucide-react';
import { FileUpload } from '../files/FileUpload';
import './Chat.css';

export const ChatView: React.FC = () => {
  const { messages, currentChannel, sendMessage } = useAppStore();
  const { user } = useAuthStore();
  const [input, setInput] = useState('');
  const messagesEndRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    messagesEndRef.current?.scrollIntoView({ behavior: 'smooth' });
  }, [messages]);

  const handleSend = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!input.trim() || !currentChannel) return;
    await sendMessage(currentChannel.id, input.trim());
    setInput('');
  };

  const formatTime = (dateStr: string) => {
    const date = new Date(dateStr);
    return date.toLocaleTimeString('fr-FR', { hour: '2-digit', minute: '2-digit' });
  };

  const formatDate = (dateStr: string) => {
    const date = new Date(dateStr);
    const today = new Date();
    if (date.toDateString() === today.toDateString()) return "Aujourd'hui";
    const yesterday = new Date(today);
    yesterday.setDate(yesterday.getDate() - 1);
    if (date.toDateString() === yesterday.toDateString()) return 'Hier';
    return date.toLocaleDateString('fr-FR', { day: 'numeric', month: 'long', year: 'numeric' });
  };

  if (!currentChannel) {
    return (
      <div className="chat-empty">
        <div className="chat-empty-content">
          <h2>Bienvenue sur Vox</h2>
          <p>Sélectionnez un canal pour commencer à discuter</p>
        </div>
      </div>
    );
  }

  // Grouper les messages par date
  let lastDate = '';

  return (
    <div className="chat-view">
      {/* Header du canal */}
      <div className="chat-header">
        <span className="chat-header-hash">#</span>
        <h3>{currentChannel.name}</h3>
        {currentChannel.description && (
          <span className="chat-header-desc">{currentChannel.description}</span>
        )}
      </div>

      {/* Zone des messages */}
      <div className="chat-messages">
        {messages.map((msg, idx) => {
          const dateLabel = formatDate(msg.created_at);
          const showDateSeparator = dateLabel !== lastDate;
          lastDate = dateLabel;

          // Grouper les messages consécutifs du même auteur
          const prevMsg = messages[idx - 1];
          const isGrouped = prevMsg
            && prevMsg.author_id === msg.author_id
            && !showDateSeparator
            && new Date(msg.created_at).getTime() - new Date(prevMsg.created_at).getTime() < 5 * 60 * 1000;

          return (
            <React.Fragment key={msg.id}>
              {showDateSeparator && (
                <div className="date-separator">
                  <span>{dateLabel}</span>
                </div>
              )}
              <div className={`message ${isGrouped ? 'grouped' : ''}`}>
                {!isGrouped && (
                  <div className="message-avatar">
                    {msg.author_avatar_url ? (
                      <img src={msg.author_avatar_url} alt={msg.author_display_name} />
                    ) : (
                      <div className="avatar-placeholder">
                        {msg.author_display_name.charAt(0).toUpperCase()}
                      </div>
                    )}
                  </div>
                )}
                <div className={`message-body ${isGrouped ? 'grouped-body' : ''}`}>
                  {!isGrouped && (
                    <div className="message-header">
                      <span className="message-author">{msg.author_display_name}</span>
                      <span className="message-time">{formatTime(msg.created_at)}</span>
                    </div>
                  )}
                  <div className="message-content">
                    {isGrouped && (
                      <span className="grouped-time">{formatTime(msg.created_at)}</span>
                    )}
                    <p>{msg.content}</p>
                    {msg.edited && <span className="edited-tag">(modifié)</span>}
                  </div>
                </div>
                {msg.author_id === user?.id && (
                  <div className="message-actions">
                    <button title="Modifier"><Edit3 size={14} /></button>
                    <button title="Supprimer"><Trash2 size={14} /></button>
                  </div>
                )}
              </div>
            </React.Fragment>
          );
        })}
        <div ref={messagesEndRef} />
      </div>

      {/* Zone de saisie */}
      <form onSubmit={handleSend} className="chat-input-bar">
        <FileUpload channelId={currentChannel.id} />
        <input
          type="text"
          value={input}
          onChange={(e) => setInput(e.target.value)}
          placeholder={`Écrire dans #${currentChannel.name}...`}
          autoFocus
        />
        <button type="submit" disabled={!input.trim()}>
          <Send size={18} />
        </button>
      </form>
    </div>
  );
};
