import React from 'react';
import './SettingsScreen.css';

const SettingsScreen: React.FC = () => {
  return (
    <div className="settings-screen">
      <header className="settings-header">
        <h1>Настройки</h1>
      </header>
      <div className="settings-list">
        <div className="settings-item">Профиль</div>
        <div className="settings-item">Уведомления</div>
        <div className="settings-item">Конфиденциальность</div>
        <div className="settings-item">Внешний вид</div>
        <div className="settings-item">О приложении</div>
      </div>
    </div>
  );
};

export default SettingsScreen;
