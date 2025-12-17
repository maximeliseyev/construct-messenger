import React, { useState } from 'react';
import ChatListScreen from './components/ChatListScreen';
import SettingsScreen from './components/SettingsScreen';
import ChatScreen from './components/ChatScreen';
import CryptoDemo from './components/CryptoDemo';
import NavigationBar from './components/NavigationBar';
import './App.css';

type Screen = 'chats' | 'settings' | 'chat' | 'crypto';

function App() {
  const [currentScreen, setCurrentScreen] = useState<Screen>('crypto');
  const [selectedChatId, setSelectedChatId] = useState<string | null>(null);

  const navigateToChat = (chatId: string) => {
    setSelectedChatId(chatId);
    setCurrentScreen('chat');
  };

  const navigateBack = () => {
    setSelectedChatId(null);
    setCurrentScreen('chats');
  }

  const renderScreen = () => {
    switch (currentScreen) {
      case 'chats':
        return <ChatListScreen onChatSelect={navigateToChat} />;
      case 'settings':
        return <SettingsScreen />;
      case 'chat':
        return <ChatScreen chatId={selectedChatId!} onBack={navigateBack} />;
      case 'crypto':
        return <CryptoDemo />;
      default:
        return <ChatListScreen onChatSelect={navigateToChat} />;
    }
  };

  return (
    <div className="app-container">
      <main className="main-content">
        {renderScreen()}
      </main>
      {currentScreen !== 'chat' && (
        <NavigationBar currentScreen={currentScreen} onNavigate={setCurrentScreen} />
      )}
    </div>
  );
}

export default App;
