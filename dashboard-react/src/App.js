import React, { useState } from 'react';
import { BrowserRouter as Router, Routes, Route } from 'react-router-dom';
import { Layout, Menu, theme, ConfigProvider } from 'antd';
import {
  DashboardOutlined,
  DesktopOutlined,
  AppstoreOutlined,
  GatewayOutlined,
  MonitorOutlined,
} from '@ant-design/icons';
import Dashboard from './components/Dashboard';
import DeviceManagement from './components/DeviceManagement';
import ApplicationManagement from './components/ApplicationManagement';
import GatewayManagement from './components/GatewayManagement';
import Monitoring from './components/Monitoring';
import './App.css';

const { Header, Sider, Content } = Layout;

function App() {
  const [collapsed, setCollapsed] = useState(false);
  const {
    token: { colorBgContainer, borderRadiusLG },
  } = theme.useToken();

  const menuItems = [
    {
      key: 'dashboard',
      icon: <DashboardOutlined />,
      label: 'Dashboard',
    },
    {
      key: 'devices',
      icon: <DesktopOutlined />,
      label: 'Device Management',
    },
    {
      key: 'applications',
      icon: <AppstoreOutlined />,
      label: 'Application Management',
    },
    {
      key: 'gateways',
      icon: <GatewayOutlined />,
      label: 'Gateway Management',
    },
    {
      key: 'monitoring',
      icon: <MonitorOutlined />,
      label: 'Monitoring',
    },
  ];

  return (
    <ConfigProvider
      theme={{
        token: {
          colorPrimary: '#1890ff',
          borderRadius: 6,
        },
      }}
    >
      <Router>
        <Layout style={{ minHeight: '100vh' }}>
          <Sider 
            trigger={null} 
            collapsible 
            collapsed={collapsed}
            style={{
              background: colorBgContainer,
            }}
          >
            <div className="logo" style={{ 
              height: 32, 
              margin: 16, 
              background: 'rgba(255, 255, 255, 0.3)',
              borderRadius: borderRadiusLG,
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'center',
              fontWeight: 'bold',
              color: '#1890ff'
            }}>
              {collapsed ? 'W' : 'Wasmbed'}
            </div>
            <Menu
              theme="light"
              mode="inline"
              defaultSelectedKeys={['dashboard']}
              items={menuItems}
              onClick={({ key }) => {
                // Navigation will be handled by React Router
                window.history.pushState({}, '', `/${key}`);
                window.dispatchEvent(new PopStateEvent('popstate'));
              }}
            />
          </Sider>
          <Layout>
            <Header
              style={{
                padding: 0,
                background: colorBgContainer,
                display: 'flex',
                alignItems: 'center',
                justifyContent: 'space-between',
                paddingRight: 24,
              }}
            >
              <div style={{ display: 'flex', alignItems: 'center' }}>
                <button
                  onClick={() => setCollapsed(!collapsed)}
                  style={{
                    fontSize: '16px',
                    width: 64,
                    height: 64,
                    border: 'none',
                    background: 'transparent',
                    cursor: 'pointer',
                  }}
                >
                  {collapsed ? '☰' : '✕'}
                </button>
                <h1 style={{ margin: 0, marginLeft: 16, fontSize: '20px', fontWeight: 'bold' }}>
                  Wasmbed Platform Dashboard
                </h1>
              </div>
              <div style={{ color: '#666', fontSize: '14px' }}>
                Edge Device Management Platform
              </div>
            </Header>
            <Content
              style={{
                margin: '24px 16px',
                padding: 24,
                minHeight: 280,
                background: colorBgContainer,
                borderRadius: borderRadiusLG,
              }}
            >
              <Routes>
                <Route path="/" element={<Dashboard />} />
                <Route path="/dashboard" element={<Dashboard />} />
                <Route path="/devices" element={<DeviceManagement />} />
                <Route path="/applications" element={<ApplicationManagement />} />
                <Route path="/gateways" element={<GatewayManagement />} />
                <Route path="/monitoring" element={<Monitoring />} />
              </Routes>
            </Content>
          </Layout>
        </Layout>
      </Router>
    </ConfigProvider>
  );
}

export default App;
