import React, { useState } from 'react';
import { useAppStore } from '../../stores/appStore';
import { X } from 'lucide-react';
import './Modal.css';

interface Props {
  isOpen: boolean;
  onClose: () => void;
  spaceId: string;
}

export const CreateChannelModal: React.FC<Props> = ({ isOpen, onClose, spaceId }) => {
  const [name, setName] = useState('');
  const [description, setDescription] = useState('');
  const [channelType, setChannelType] = useState('Text');
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState('');
  const { createChannel } = useAppStore();

  if (!isOpen) return null;

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!name.trim()) return;
    setIsLoading(true);
    setError('');
    try {
      await createChannel(spaceId, name.trim(), description.trim() || undefined, channelType);
      setName('');
      setDescription('');
      setChannelType('Text');
      onClose();
    } catch (err: any) {
      setError(err.response?.data?.error?.message || 'Erreur lors de la création');
    } finally {
      setIsLoading(false);
    }
  };

  return (
    <div className="modal-overlay" onClick={onClose}>
      <div className="modal-content" onClick={(e) => e.stopPropagation()}>
        <div className="modal-header">
          <h2>Créer un canal</h2>
          <button className="modal-close" onClick={onClose}>
            <X size={20} />
          </button>
        </div>

        <form onSubmit={handleSubmit} className="modal-form">
          {error && <div className="modal-error">{error}</div>}

          <div className="modal-field">
            <label htmlFor="channel-type">Type de canal</label>
            <div className="channel-type-select">
              <button
                type="button"
                className={`type-option ${channelType === 'Text' ? 'active' : ''}`}
                onClick={() => setChannelType('Text')}
              >
                # Textuel
              </button>
              <button
                type="button"
                className={`type-option ${channelType === 'Voice' ? 'active' : ''}`}
                onClick={() => setChannelType('Voice')}
              >
                🔊 Vocal
              </button>
            </div>
          </div>

          <div className="modal-field">
            <label htmlFor="channel-name">Nom du canal</label>
            <input
              id="channel-name"
              type="text"
              value={name}
              onChange={(e) => setName(e.target.value)}
              placeholder={channelType === 'Text' ? 'général' : 'Salon vocal'}
              required
              autoFocus
              maxLength={100}
            />
          </div>

          <div className="modal-field">
            <label htmlFor="channel-desc">Description (optionnel)</label>
            <input
              id="channel-desc"
              type="text"
              value={description}
              onChange={(e) => setDescription(e.target.value)}
              placeholder="De quoi parle ce canal ?"
              maxLength={500}
            />
          </div>

          <div className="modal-actions">
            <button type="button" className="btn-secondary" onClick={onClose}>
              Annuler
            </button>
            <button type="submit" className="btn-primary" disabled={isLoading || !name.trim()}>
              {isLoading ? 'Création...' : 'Créer'}
            </button>
          </div>
        </form>
      </div>
    </div>
  );
};
