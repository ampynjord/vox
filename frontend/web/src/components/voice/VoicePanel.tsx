import React, { useState, useEffect, useCallback, useRef } from 'react';
import { Mic, MicOff, Headphones, HeadphoneOff, PhoneOff, Monitor, Users } from 'lucide-react';
import { wsService } from '../../services/websocket';
import { voiceService } from '../../services/data';
import type { VoiceParticipant, WsServerEvent } from '../../types';
import './Voice.css';

interface VoicePanelProps {
  channelId: string;
  channelName: string;
}

export const VoicePanel: React.FC<VoicePanelProps> = ({ channelId, channelName }) => {
  const [isConnected, setIsConnected] = useState(false);
  const [isMuted, setIsMuted] = useState(false);
  const [isDeafened, setIsDeafened] = useState(false);
  const [isScreenSharing, setIsScreenSharing] = useState(false);
  const [participants, setParticipants] = useState<VoiceParticipant[]>([]);
  const localStreamRef = useRef<MediaStream | null>(null);
  const peerConnectionsRef = useRef<Map<string, RTCPeerConnection>>(new Map());

  const ICE_SERVERS: RTCConfiguration = {
    iceServers: [
      { urls: 'stun:stun.l.google.com:19302' },
      { urls: 'stun:stun1.l.google.com:19302' },
    ],
  };

  // Charger les participants au montage
  useEffect(() => {
    voiceService.getParticipants(channelId).then(setParticipants).catch(console.error);
  }, [channelId]);

  // Écouter les événements vocaux du WebSocket
  useEffect(() => {
    const handleVoiceJoined = (event: WsServerEvent) => {
      if (event.data?.channel_id === channelId) {
        setParticipants((prev) => {
          const exists = prev.some((p) => p.user_id === event.data.user_id);
          if (exists) return prev;
          return [...prev, {
            user_id: event.data.user_id,
            username: event.data.username,
            is_muted: false,
            is_deafened: false,
            is_screen_sharing: false,
          }];
        });

        // Si on est connecté, initier une connexion WebRTC avec le nouvel arrivant
        if (isConnected && localStreamRef.current) {
          createPeerConnection(event.data.user_id, true);
        }
      }
    };

    const handleVoiceLeft = (event: WsServerEvent) => {
      if (event.data?.channel_id === channelId) {
        setParticipants((prev) => prev.filter((p) => p.user_id !== event.data.user_id));
        // Fermer la connexion WebRTC
        closePeerConnection(event.data.user_id);
      }
    };

    const handleVoiceState = (event: WsServerEvent) => {
      if (event.data?.channel_id === channelId) {
        setParticipants((prev) =>
          prev.map((p) =>
            p.user_id === event.data.user_id
              ? {
                  ...p,
                  is_muted: event.data.is_muted,
                  is_deafened: event.data.is_deafened,
                  is_screen_sharing: event.data.is_screen_sharing,
                }
              : p
          )
        );
      }
    };

    const handleVoiceSignal = async (event: WsServerEvent) => {
      const { from_user_id, signal_type, sdp, candidate, sdp_mid, sdp_m_line_index } = event.data;

      if (signal_type === 'offer') {
        const pc = createPeerConnection(from_user_id, false);
        await pc.setRemoteDescription(new RTCSessionDescription({ type: 'offer', sdp }));
        const answer = await pc.createAnswer();
        await pc.setLocalDescription(answer);
        wsService.sendVoiceSignal(from_user_id, 'answer', answer.sdp);
      } else if (signal_type === 'answer') {
        const pc = peerConnectionsRef.current.get(from_user_id);
        if (pc) {
          await pc.setRemoteDescription(new RTCSessionDescription({ type: 'answer', sdp }));
        }
      } else if (signal_type === 'ice-candidate' && candidate) {
        const pc = peerConnectionsRef.current.get(from_user_id);
        if (pc) {
          await pc.addIceCandidate(new RTCIceCandidate({
            candidate,
            sdpMid: sdp_mid,
            sdpMLineIndex: sdp_m_line_index,
          }));
        }
      }
    };

    wsService.on('VoiceUserJoined', handleVoiceJoined);
    wsService.on('VoiceUserLeft', handleVoiceLeft);
    wsService.on('VoiceStateUpdate', handleVoiceState);
    wsService.on('VoiceSignal', handleVoiceSignal);

    return () => {
      wsService.off('VoiceUserJoined', handleVoiceJoined);
      wsService.off('VoiceUserLeft', handleVoiceLeft);
      wsService.off('VoiceStateUpdate', handleVoiceState);
      wsService.off('VoiceSignal', handleVoiceSignal);
    };
  }, [channelId, isConnected]);

  const createPeerConnection = useCallback(
    (userId: string, isInitiator: boolean): RTCPeerConnection => {
      // Fermer l'ancienne connexion si elle existe
      closePeerConnection(userId);

      const pc = new RTCPeerConnection(ICE_SERVERS);
      peerConnectionsRef.current.set(userId, pc);

      // Ajouter notre stream local
      if (localStreamRef.current) {
        localStreamRef.current.getTracks().forEach((track) => {
          pc.addTrack(track, localStreamRef.current!);
        });
      }

      // ICE candidates
      pc.onicecandidate = (event) => {
        if (event.candidate) {
          wsService.sendVoiceSignal(
            userId,
            'ice-candidate',
            undefined,
            event.candidate.candidate,
            event.candidate.sdpMid ?? undefined,
            event.candidate.sdpMLineIndex ?? undefined,
          );
        }
      };

      // Stream distant reçu
      pc.ontrack = (event) => {
        const audio = new Audio();
        audio.srcObject = event.streams[0];
        audio.autoplay = true;
        audio.id = `voice-audio-${userId}`;
        document.body.appendChild(audio);
      };

      // Si on est l'initiateur, créer l'offre
      if (isInitiator) {
        pc.createOffer().then(async (offer) => {
          await pc.setLocalDescription(offer);
          wsService.sendVoiceSignal(userId, 'offer', offer.sdp);
        });
      }

      return pc;
    },
    []
  );

  const closePeerConnection = (userId: string) => {
    const pc = peerConnectionsRef.current.get(userId);
    if (pc) {
      pc.close();
      peerConnectionsRef.current.delete(userId);
      // Retirer l'audio
      const audio = document.getElementById(`voice-audio-${userId}`);
      audio?.remove();
    }
  };

  const joinVoice = async () => {
    try {
      const stream = await navigator.mediaDevices.getUserMedia({ audio: true });
      localStreamRef.current = stream;
      setIsConnected(true);
      wsService.joinVoice(channelId);
    } catch (err) {
      console.error('Erreur accès micro:', err);
    }
  };

  const leaveVoice = () => {
    // Fermer toutes les connexions peer
    peerConnectionsRef.current.forEach((_, userId) => closePeerConnection(userId));

    // Arrêter le stream local
    localStreamRef.current?.getTracks().forEach((t) => t.stop());
    localStreamRef.current = null;

    setIsConnected(false);
    setIsMuted(false);
    setIsDeafened(false);
    setIsScreenSharing(false);
    wsService.leaveVoice(channelId);
  };

  const toggleMute = () => {
    if (localStreamRef.current) {
      const audioTrack = localStreamRef.current.getAudioTracks()[0];
      if (audioTrack) {
        audioTrack.enabled = isMuted; // inverse car on toggle
        setIsMuted(!isMuted);
        wsService.updateVoiceState(!isMuted, isDeafened, isScreenSharing);
      }
    }
  };

  const toggleDeafen = () => {
    setIsDeafened(!isDeafened);
    wsService.updateVoiceState(isMuted, !isDeafened, isScreenSharing);
    // Muter/démuter tous les audio distants
    peerConnectionsRef.current.forEach((_, userId) => {
      const audio = document.getElementById(`voice-audio-${userId}`) as HTMLAudioElement;
      if (audio) audio.muted = !isDeafened;
    });
  };

  // Cleanup au démontage
  useEffect(() => {
    return () => {
      if (isConnected) leaveVoice();
    };
  }, []);

  return (
    <div className="voice-panel">
      <div className="voice-header">
        <Users size={16} />
        <span className="voice-channel-name">{channelName}</span>
        <span className="voice-participant-count">{participants.length}</span>
      </div>

      <div className="voice-participants">
        {participants.map((p) => (
          <div key={p.user_id} className="voice-participant">
            <div className={`voice-participant-avatar ${p.is_muted ? 'muted' : ''}`}>
              {p.username.charAt(0).toUpperCase()}
            </div>
            <span className="voice-participant-name">{p.username}</span>
            <div className="voice-participant-icons">
              {p.is_muted && <MicOff size={12} />}
              {p.is_deafened && <HeadphoneOff size={12} />}
              {p.is_screen_sharing && <Monitor size={12} />}
            </div>
          </div>
        ))}
        {participants.length === 0 && (
          <div className="voice-empty">Aucun participant</div>
        )}
      </div>

      <div className="voice-controls">
        {!isConnected ? (
          <button className="voice-btn join" onClick={joinVoice}>
            <Headphones size={18} />
            Rejoindre
          </button>
        ) : (
          <>
            <button
              className={`voice-btn ${isMuted ? 'active' : ''}`}
              onClick={toggleMute}
              title={isMuted ? 'Activer le micro' : 'Couper le micro'}
            >
              {isMuted ? <MicOff size={18} /> : <Mic size={18} />}
            </button>
            <button
              className={`voice-btn ${isDeafened ? 'active' : ''}`}
              onClick={toggleDeafen}
              title={isDeafened ? 'Activer le son' : 'Couper le son'}
            >
              {isDeafened ? <HeadphoneOff size={18} /> : <Headphones size={18} />}
            </button>
            <button className="voice-btn disconnect" onClick={leaveVoice} title="Quitter">
              <PhoneOff size={18} />
            </button>
          </>
        )}
      </div>
    </div>
  );
};
