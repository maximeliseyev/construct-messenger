import React from 'react';
import './ChatListScreen.css';

type ChatListScreenProps = {
  onChatSelect: (chatId: string) => void;
};

const chats = [
  { id: '1', name: 'Alice' },
  { id: '2', name: 'Bob' },
  { id: '3', name: 'Charlie' },
];

const ChatListScreen: React.FC<ChatListScreenProps> = ({ onChatSelect }) => {
  return (
    <div className="chat-list-screen">
      <header className="chat-list-header">
        <h1>Чаты</h1>
      </header>
      <div className="chat-list">
        {chats.map(chat => (
          <div key={chat.id} className="chat-list-item" onClick={() => onChatSelect(chat.id)}>
            <div className="chat-info">
              <span className="chat-name">{chat.name}</span>
              <span className="last-message">Last message placeholder...</span>
            </div>
            <span className="chat-timestamp">10:42</span>
          </div>
        ))}
      </div>
    </div>
  );
};

export default ChatListScreen;
