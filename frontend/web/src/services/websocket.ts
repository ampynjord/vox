import type { WsClientEvent, WsServerEvent } from '../types';

const GATEWAY_URL = import.meta.env.VITE_WS_URL || import.meta.env.VITE_GATEWAY_URL || 'ws://localhost:8081/ws';

type EventHandler = (event: WsServerEvent) => void;

class WebSocketService {
  private ws: WebSocket | null = null;
  private handlers: Map<string, Set<EventHandler>> = new Map();
  private reconnectAttempts = 0;
  private maxReconnectAttempts = 10;
  private reconnectDelay = 1000;
  private token: string | null = null;

  connect(token: string) {
    this.token = token;
    this.reconnectAttempts = 0;
    this.createConnection();
  }

  private createConnection() {
    if (!this.token) return;

    const url = `${GATEWAY_URL}?token=${this.token}`;
    this.ws = new WebSocket(url);

    this.ws.onopen = () => {
      console.log('[Vox] Connecté au gateway');
      this.reconnectAttempts = 0;
    };

    this.ws.onmessage = (event) => {
      try {
        const serverEvent: WsServerEvent = JSON.parse(event.data);
        this.emit(serverEvent.type, serverEvent);
      } catch (e) {
        console.error('[Vox] Message invalide:', e);
      }
    };

    this.ws.onclose = (event) => {
      console.log('[Vox] Déconnecté du gateway', event.code);
      if (this.reconnectAttempts < this.maxReconnectAttempts) {
        const delay = this.reconnectDelay * Math.pow(2, this.reconnectAttempts);
        console.log(`[Vox] Reconnexion dans ${delay}ms...`);
        setTimeout(() => {
          this.reconnectAttempts++;
          this.createConnection();
        }, delay);
      }
    };

    this.ws.onerror = (error) => {
      console.error('[Vox] Erreur WebSocket:', error);
    };
  }

  disconnect() {
    this.maxReconnectAttempts = 0;
    this.ws?.close();
    this.ws = null;
  }

  send(event: WsClientEvent) {
    if (this.ws?.readyState === WebSocket.OPEN) {
      this.ws.send(JSON.stringify(event));
    }
  }

  subscribe(channelId: string) {
    this.send({ type: 'Subscribe', data: { channel_id: channelId } });
  }

  unsubscribe(channelId: string) {
    this.send({ type: 'Unsubscribe', data: { channel_id: channelId } });
  }

  sendMessage(channelId: string, content: string, replyTo?: string) {
    this.send({
      type: 'SendMessage',
      data: { channel_id: channelId, content, reply_to: replyTo },
    });
  }

  startTyping(channelId: string) {
    this.send({ type: 'StartTyping', data: { channel_id: channelId } });
  }

  // ========== Voice ==========

  joinVoice(channelId: string) {
    this.send({ type: 'JoinVoice', data: { channel_id: channelId } });
  }

  leaveVoice(channelId: string) {
    this.send({ type: 'LeaveVoice', data: { channel_id: channelId } });
  }

  updateVoiceState(isMuted?: boolean, isDeafened?: boolean, isScreenSharing?: boolean) {
    this.send({
      type: 'UpdateVoiceState',
      data: {
        is_muted: isMuted,
        is_deafened: isDeafened,
        is_screen_sharing: isScreenSharing,
      },
    });
  }

  sendVoiceSignal(
    targetUserId: string,
    signalType: string,
    sdp?: string,
    candidate?: string,
    sdpMid?: string,
    sdpMLineIndex?: number,
  ) {
    this.send({
      type: 'VoiceSignal',
      data: {
        target_user_id: targetUserId,
        signal_type: signalType,
        sdp,
        candidate,
        sdp_mid: sdpMid,
        sdp_m_line_index: sdpMLineIndex,
      },
    });
  }

  on(eventType: string, handler: EventHandler) {
    if (!this.handlers.has(eventType)) {
      this.handlers.set(eventType, new Set());
    }
    this.handlers.get(eventType)!.add(handler);
  }

  off(eventType: string, handler: EventHandler) {
    this.handlers.get(eventType)?.delete(handler);
  }

  private emit(eventType: string, event: WsServerEvent) {
    this.handlers.get(eventType)?.forEach((handler) => handler(event));
    // Aussi émettre vers les handlers génériques '*'
    this.handlers.get('*')?.forEach((handler) => handler(event));
  }
}

export const wsService = new WebSocketService();
