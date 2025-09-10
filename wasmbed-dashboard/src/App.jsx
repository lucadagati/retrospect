import React, { useState, useEffect } from 'react';
import { ThemeProvider, createTheme } from '@mui/material/styles';
import { CssBaseline } from '@mui/material';
import DashboardLayout from './components/DashboardLayout';
import OverviewDashboard from './components/OverviewDashboard';
import DroneControlDashboard from './components/DroneControlDashboard';
import DevicesDashboard from './components/DevicesDashboard';
import ApplicationsDashboard from './components/ApplicationsDashboard';
import MonitoringDashboard from './components/MonitoringDashboard';
import SettingsDashboard from './components/SettingsDashboard';

// Tema migliorato
const theme = createTheme({
  palette: {
    mode: 'light',
    primary: {
      main: '#1976d2',
      light: '#42a5f5',
      dark: '#1565c0',
    },
    secondary: {
      main: '#dc004e',
    },
    background: {
      default: '#f5f5f5',
      paper: '#ffffff',
    },
    success: {
      main: '#4caf50',
    },
    warning: {
      main: '#ff9800',
    },
    error: {
      main: '#f44336',
    },
  },
  typography: {
    fontFamily: 'Roboto, sans-serif',
    h6: {
      fontWeight: 600,
    },
  },
  components: {
    MuiCard: {
      styleOverrides: {
        root: {
          boxShadow: '0 2px 8px rgba(0,0,0,0.1)',
          borderRadius: '12px',
          transition: 'all 0.3s ease-in-out',
          '&:hover': {
            boxShadow: '0 4px 16px rgba(0,0,0,0.15)',
            transform: 'translateY(-2px)',
          },
        },
      },
    },
    MuiButton: {
      styleOverrides: {
        root: {
          borderRadius: '8px',
          textTransform: 'none',
          fontWeight: 500,
        },
      },
    },
  },
});

// Context per condividere dati WebSocket
export const WebSocketContext = React.createContext();

function App() {
  const [currentTab, setCurrentTab] = useState('overview');
  const [wsData, setWsData] = useState({
    devices: [],
    applications: [],
    metrics: {},
    alerts: [],
    droneStatus: {
      position: { x: 0, y: 0, z: 0 },
      attitude: { roll: 0, pitch: 0, yaw: 0 },
      battery: { percentage: 100, voltage: 12.6 },
      flightMode: 'Manual',
      armed: false,
      connected: true
    }
  });
  const [connectionStatus, setConnectionStatus] = useState('disconnected');

  // Simulazione WebSocket migliorata
  useEffect(() => {
    let interval;
    let wsSimulation;

    const startWebSocketSimulation = () => {
      setConnectionStatus('connecting');
      
      setTimeout(() => {
        setConnectionStatus('connected');
        
        wsSimulation = setInterval(() => {
          setWsData(prevData => ({
            ...prevData,
            metrics: {
              cpu: 20 + Math.random() * 60,
              memory: 30 + Math.random() * 50,
              timestamp: new Date().toISOString()
            },
            droneStatus: {
              ...prevData.droneStatus,
              position: {
                x: prevData.droneStatus.position.x + (Math.random() - 0.5) * 0.1,
                y: prevData.droneStatus.position.y + (Math.random() - 0.5) * 0.1,
                z: Math.max(0, prevData.droneStatus.position.z + (Math.random() - 0.5) * 0.05)
              },
              attitude: {
                roll: (Math.random() - 0.5) * 0.1,
                pitch: (Math.random() - 0.5) * 0.1,
                yaw: prevData.droneStatus.attitude.yaw + (Math.random() - 0.5) * 0.05
              },
              battery: {
                ...prevData.droneStatus.battery,
                percentage: Math.max(0, prevData.droneStatus.battery.percentage - 0.002)
              }
            }
          }));
        }, 100);
      }, 1000);
    };

    startWebSocketSimulation();

    return () => {
      if (wsSimulation) clearInterval(wsSimulation);
      if (interval) clearInterval(interval);
    };
  }, []);

  const renderContent = () => {
    switch (currentTab) {
      case 'overview':
        return <OverviewDashboard />;
      case 'drone':
        return <DroneControlDashboard />;
      case 'devices':
        return <DevicesDashboard />;
      case 'applications':
        return <ApplicationsDashboard />;
      case 'monitoring':
        return <MonitoringDashboard />;
      case 'settings':
        return <SettingsDashboard />;
      default:
        return <OverviewDashboard />;
    }
  };

  return (
    <ThemeProvider theme={theme}>
      <CssBaseline />
      <WebSocketContext.Provider value={{ wsData, connectionStatus, setWsData }}>
        <DashboardLayout 
          selectedTab={currentTab} 
          onTabChange={setCurrentTab}
          connectionStatus={connectionStatus}
        >
          {renderContent()}
        </DashboardLayout>
      </WebSocketContext.Provider>
    </ThemeProvider>
  );
}

export default App;