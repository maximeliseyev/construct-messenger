import React from 'react';
import './NavigationBar.css';

type NavigationBarProps = {
  currentScreen: string;
  onNavigate: (screen: 'chats' | 'settings') => void;
};

const NavigationBar: React.FC<NavigationBarProps> = ({ currentScreen, onNavigate }) => {
  return (
    <nav className="navigation-bar">
      <button
        className={currentScreen === 'chats' ? 'active' : ''}
        onClick={() => onNavigate('chats')}
      >
        Чаты
      </button>
      <button
        className={currentScreen === 'settings' ? 'active' : ''}
        onClick={() => onNavigate('settings')}
      >
        Настройки
      </button>
    </nav>
  );
};

export default NavigationBar;
