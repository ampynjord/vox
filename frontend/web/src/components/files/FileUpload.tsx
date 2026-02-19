import React, { useRef, useState } from 'react';
import { Paperclip, X, Upload, FileText, Image, Film, Music } from 'lucide-react';
import { fileService } from '../../services/data';
import type { FileInfo } from '../../types';
import './Files.css';

interface FileUploadProps {
  channelId: string;
  onUploaded?: (file: FileInfo) => void;
}

export const FileUpload: React.FC<FileUploadProps> = ({ channelId, onUploaded }) => {
  const [isUploading, setIsUploading] = useState(false);
  const [progress, setProgress] = useState(0);
  const [pendingFile, setPendingFile] = useState<File | null>(null);
  const inputRef = useRef<HTMLInputElement>(null);

  const MAX_SIZE = 50 * 1024 * 1024; // 50 Mo

  const handleFileSelect = (e: React.ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files?.[0];
    if (!file) return;

    if (file.size > MAX_SIZE) {
      alert('Fichier trop volumineux (max 50 Mo)');
      return;
    }

    setPendingFile(file);
  };

  const handleUpload = async () => {
    if (!pendingFile) return;

    setIsUploading(true);
    setProgress(0);

    try {
      // Simuler la progression (l'API ne renvoie pas de progression)
      const progressInterval = setInterval(() => {
        setProgress((p) => Math.min(p + 10, 90));
      }, 200);

      const uploaded = await fileService.upload(channelId, pendingFile);

      clearInterval(progressInterval);
      setProgress(100);

      onUploaded?.(uploaded);
      setPendingFile(null);
    } catch (err) {
      console.error('Erreur upload:', err);
    } finally {
      setIsUploading(false);
      setProgress(0);
    }
  };

  const cancelPending = () => {
    setPendingFile(null);
    if (inputRef.current) inputRef.current.value = '';
  };

  const getFileIcon = (mimeType: string) => {
    if (mimeType.startsWith('image/')) return <Image size={16} />;
    if (mimeType.startsWith('video/')) return <Film size={16} />;
    if (mimeType.startsWith('audio/')) return <Music size={16} />;
    return <FileText size={16} />;
  };

  const formatSize = (bytes: number) => {
    if (bytes < 1024) return `${bytes} o`;
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} Ko`;
    return `${(bytes / (1024 * 1024)).toFixed(1)} Mo`;
  };

  return (
    <div className="file-upload">
      <input
        ref={inputRef}
        type="file"
        onChange={handleFileSelect}
        style={{ display: 'none' }}
      />

      <button
        className="file-upload-trigger"
        onClick={() => inputRef.current?.click()}
        title="Joindre un fichier"
        disabled={isUploading}
      >
        <Paperclip size={18} />
      </button>

      {pendingFile && (
        <div className="file-preview">
          <div className="file-preview-info">
            {getFileIcon(pendingFile.type)}
            <span className="file-preview-name">{pendingFile.name}</span>
            <span className="file-preview-size">{formatSize(pendingFile.size)}</span>
          </div>

          {isUploading ? (
            <div className="file-progress">
              <div className="file-progress-bar" style={{ width: `${progress}%` }} />
            </div>
          ) : (
            <div className="file-preview-actions">
              <button className="file-btn send" onClick={handleUpload}>
                <Upload size={14} /> Envoyer
              </button>
              <button className="file-btn cancel" onClick={cancelPending}>
                <X size={14} />
              </button>
            </div>
          )}
        </div>
      )}
    </div>
  );
};

// Composant pour afficher un fichier joint dans un message
interface FileAttachmentProps {
  file: FileInfo;
}

export const FileAttachment: React.FC<FileAttachmentProps> = ({ file }) => {
  const isImage = file.mime_type.startsWith('image/');
  const isVideo = file.mime_type.startsWith('video/');
  const isAudio = file.mime_type.startsWith('audio/');

  const formatSize = (bytes: number) => {
    if (bytes < 1024) return `${bytes} o`;
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} Ko`;
    return `${(bytes / (1024 * 1024)).toFixed(1)} Mo`;
  };

  const fileUrl = file.url || '#';

  if (isImage) {
    return (
      <div className="file-attachment image">
        <a href={fileUrl} target="_blank" rel="noopener noreferrer">
          <img src={fileUrl} alt={file.original_filename} className="file-image-preview" />
        </a>
        <div className="file-attachment-meta">
          {file.original_filename} — {formatSize(file.size_bytes)}
        </div>
      </div>
    );
  }

  if (isVideo) {
    return (
      <div className="file-attachment video">
        <video src={fileUrl} controls className="file-video-preview" />
        <div className="file-attachment-meta">
          {file.original_filename} — {formatSize(file.size_bytes)}
        </div>
      </div>
    );
  }

  if (isAudio) {
    return (
      <div className="file-attachment audio">
        <audio src={fileUrl} controls className="file-audio-player" />
        <div className="file-attachment-meta">
          {file.original_filename} — {formatSize(file.size_bytes)}
        </div>
      </div>
    );
  }

  return (
    <div className="file-attachment generic">
      <a href={fileUrl} target="_blank" rel="noopener noreferrer" className="file-attachment-link">
        <FileText size={20} />
        <div>
          <div className="file-attachment-name">{file.original_filename}</div>
          <div className="file-attachment-size">{formatSize(file.size_bytes)}</div>
        </div>
      </a>
    </div>
  );
};
