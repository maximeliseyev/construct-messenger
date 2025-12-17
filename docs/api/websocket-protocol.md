# WebSocket API Protocol v2.0

**Protocol Version**: 2.0
**Serialization**: MessagePack

This document describes the WebSocket communication protocol for the Construct Messenger.

## 1. Connection

### WebSocket Endpoint
The client connects to the server via a WebSocket endpoint. The connection must use the `wss://` protocol in production.

```typescript
const API_URL = process.env.NODE_ENV === 'production'
  ? 'wss://api.construct-messenger.com'
  : 'ws://localhost:3000';

const ws = new WebSocket(API_URL);
ws.binaryType = 'arraybuffer'; // Required for MessagePack
```

### Connection Lifecycle
1.  **Connect**: The client establishes a WebSocket connection.
2.  **Authenticate**: The client sends a `login` or `connect` message.
3.  **Communicate**: Client and server exchange messages.
4.  **Heartbeat**: The client and server may exchange ping/pong frames to keep the connection alive (standard WebSocket mechanism).
5.  **Disconnect**: The connection is closed. The client should implement a reconnection strategy with exponential backoff.

### WebSocket Manager Example
```typescript
// network/websocket-manager.ts
export class WebSocketManager {
  private ws: WebSocket | null = null;
  private reconnectAttempts = 0;

  constructor(private url: string) {}

  async connect(): Promise<void> {
    this.ws = new WebSocket(this.url);
    this.ws.binaryType = 'arraybuffer';

    this.ws.onopen = () => {
      console.log('WebSocket connected');
      this.reconnectAttempts = 0;
    };

    this.ws.onmessage = this.handleMessage.bind(this);

    this.ws.onclose = () => {
      this.handleReconnect();
    };
  }

  private handleMessage(event: MessageEvent) {
    const message = MessagePackSerializer.deserialize<ServerMessage>(event.data);
    // ... dispatch message
  }

  send(message: ClientMessage): void {
    if (!this.ws || this.ws.readyState !== WebSocket.OPEN) {
      throw new Error('WebSocket is not connected');
    }
    const buffer = MessagePackSerializer.serialize(message);
    this.ws.send(buffer);
  }

  private handleReconnect() {
    // ... implementation with exponential backoff
  }
}
```

## 2. Message Format

All messages sent over WebSocket are serialized using MessagePack. The messages follow an internally tagged enum format in TypeScript/Rust.

### Client-to-Server Messages (`ClientMessage`)
```typescript
export type ClientMessage =
  | { type: 'register'; payload: RegisterData }
  | { type: 'login'; payload: LoginData }
  | { type: 'connect'; payload: ConnectData }
  | { type: 'searchUsers'; payload: SearchUsersData }
  | { type: 'getPublicKey'; payload: GetPublicKeyData }
  | { type: 'sendMessage'; payload: ChatMessage }
  | { type: 'rotatePrekey'; payload: RotatePrekeyData }
  | { type: 'logout'; payload: LogoutData };
```

### Server-to-Client Messages (`ServerMessage`)
```typescript
export type ServerMessage =
  | { type: 'registerSuccess'; payload: RegisterSuccessData }
  | { type: 'loginSuccess'; payload: LoginSuccessData }
  | { type: 'connectSuccess'; payload: ConnectSuccessData }
  | { type: 'sessionExpired' }
  | { type: 'searchResults'; payload: SearchResultsData }
  | { type: 'publicKeyBundle'; payload: PublicKeyBundleData }
  | { type: 'message'; payload: ChatMessage }
  | { type: 'ack'; payload: AckData }
  | { type: 'keyRotationSuccess' }
  | { type: 'error'; payload: ErrorData }
  | { type: 'logoutSuccess' };
```

### Serialization
```typescript
// utils/serialization.ts
import { encode, decode } from ' @msgpack/msgpack';

export class MessagePackSerializer {
  static serialize<T>(data: T): ArrayBuffer {
    return encode(data);
  }

  static deserialize<T>(buffer: ArrayBuffer): T {
    return decode(new Uint8Array(buffer)) as T;
  }
}
```
