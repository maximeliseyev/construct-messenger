import React from 'react';
import { useDeviceType } from './hooks/useDeviceType';
import MobileApp from './MobileApp';
import DesktopApp from './DesktopApp';

const App: React.FC = () => {
  const deviceType = useDeviceType();

  if (deviceType === 'desktop') {
    return <DesktopApp />;
  }

  return <MobileApp />;
};

export default App;
