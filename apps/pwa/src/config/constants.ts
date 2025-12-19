// Application configuration constants

// Server configuration
export const SERVER_URL = import.meta.env.VITE_SERVER_URL || 'https://construct-messenger.fly.dev';
export const WS_URL = SERVER_URL.replace(/^http/, 'ws');

// API endpoints
export const API_ENDPOINTS = {
  register: '/api/register',
  message: '/api/message',
  getMessages: '/api/messages',
  getContacts: '/api/contacts',
} as const;

// WebSocket configuration
export const WS_CONFIG = {
  reconnectInterval: 3000, // milliseconds
  maxReconnectAttempts: 5,
  heartbeatInterval: 30000, // 30 seconds
} as const;

// Application metadata
export const APP_VERSION = '0.1.0';
export const APP_NAME = 'Construct Messenger';
