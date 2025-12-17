import React from 'react';
import './ChatScreen.css';

type ChatScreenProps = {
  chatId: string;
  onBack: () => void;
};

const messages = [
  { id: '1', text: 'Hey there!', sender: 'Alice' },
  { id: '2', text: 'Hello! How are you?', sender: 'Me' },
  { id: '3', text: 'Doing great, thanks!', sender: 'Alice' },
];

const ChatScreen: React.FC<ChatScreenProps> = ({ chatId, onBack }) => {
  return (
    <div className="chat-screen">
      <header className="chat-header">
        <button onClick={onBack} className="back-button">&lt;</button>
        <h1>Chat with {chatId}</h1>
      </header>
      <div className="message-list">
        {messages.map(msg => (
          <div key={msg.id} className="message-item">
            <span className="sender-name">{msg.sender}:</span>
            <span className="message-text">{msg.text}</span>
          </div>
        ))}
      </div>
      <div className="message-input-container">
        <input type="text" placeholder="Сообщение..." className="message-input" />
        <button className="send-button">Отправить</button>
      </div>
    </div>
  );
};

export default ChatScreen;
