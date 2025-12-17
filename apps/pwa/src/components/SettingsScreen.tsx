import React, { useState } from 'react';
import './SettingsScreen.css';

const SettingsScreen: React.FC = () => {
  const [serverUrl, setServerUrl] = useState('wss://construct.messenger.local');

  const handleChangeServer = () => {
    const newUrl = prompt('Enter server URL:', serverUrl);
    if (newUrl) {
      setServerUrl(newUrl);
      // TODO: Implement actual server connection logic
      console.log('Server changed to:', newUrl);
    }
  };

  const handleLogout = () => {
    if (confirm('Are you sure you want to logout?')) {
      // TODO: Implement logout logic
      console.log('Logging out...');
    }
  };

  return (
    <div className="settings-screen">
      <div className="settings-header">
        <h1 className="mono">SETTINGS</h1>
      </div>

      <div className="settings-list">
        <div className="settings-item" onClick={handleChangeServer}>
          <div className="settings-item-label mono">SERVER</div>
          <div className="settings-item-value">{serverUrl}</div>
        </div>

        <div className="settings-item logout-item" onClick={handleLogout}>
          <div className="settings-item-label mono">LOGOUT</div>
        </div>
      </div>
    </div>
  );
};

export default SettingsScreen;
