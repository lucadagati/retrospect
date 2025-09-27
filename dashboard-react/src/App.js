import React, { useState } from 'react';
import { BrowserRouter as Router, Routes, Route } from 'react-router-dom';
import { Layout, Menu, theme, ConfigProvider } from 'antd';
import {
  DashboardOutlined,
  DesktopOutlined,
  AppstoreOutlined,
  GatewayOutlined,
  MonitorOutlined,
  NodeIndexOutlined,
} from '@ant-design/icons';
import Dashboard from './components/Dashboard';
import DeviceManagement from './components/DeviceManagement';
import ApplicationManagement from './components/ApplicationManagement';
import GatewayManagement from './components/GatewayManagement';
import Monitoring from './components/Monitoring';
import NetworkTopology from './components/NetworkTopology';
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
      key: 'topology',
      icon: <NodeIndexOutlined />,
      label: 'Network Topology',
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
          borderRadius: 8,
          colorBgContainer: '#ffffff',
          colorBgElevated: '#fafafa',
          colorBorder: '#d9d9d9',
          colorText: '#262626',
          colorTextSecondary: '#8c8c8c',
          fontFamily: '-apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, "Helvetica Neue", Arial, sans-serif',
        },
        components: {
          Layout: {
            headerBg: '#ffffff',
            siderBg: '#fafafa',
          },
          Menu: {
            itemBg: 'transparent',
            itemSelectedBg: '#e6f7ff',
            itemHoverBg: '#f5f5f5',
          },
          Card: {
            borderRadius: 8,
            boxShadow: '0 1px 2px 0 rgba(0, 0, 0, 0.03), 0 1px 6px -1px rgba(0, 0, 0, 0.02), 0 2px 4px 0 rgba(0, 0, 0, 0.02)',
          },
          Table: {
            borderRadius: 8,
            headerBg: '#fafafa',
          },
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
              height: 64, 
              margin: 16, 
              background: 'rgba(255, 255, 255, 0.3)',
              borderRadius: borderRadiusLG,
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'center',
              flexDirection: 'column',
              padding: '8px'
            }}>
              <img 
                src="/Logo_Serics.png" 
                alt="MdsLab Logo" 
                style={{ 
                  height: collapsed ? 24 : 32, 
                  width: 'auto',
                  marginBottom: collapsed ? 0 : 4
                }} 
              />
              {!collapsed && (
                <div style={{ 
                  fontSize: '10px', 
                  fontWeight: 'bold', 
                  color: '#1890ff',
                  textAlign: 'center',
                  lineHeight: 1.2
                }}>
                  RETROSPECT
                  <br />
                  <span style={{ fontSize: '8px', color: '#666' }}>
                    MdsLab - UniMe
                  </span>
                </div>
              )}
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
                  RETROSPECT Dashboard
                </h1>
                <div style={{ 
                  marginLeft: 16, 
                  fontSize: '12px', 
                  color: '#666',
                  fontStyle: 'italic'
                }}>
                  secuRE inTegration middlewaRe fOr cpS in the comPutE ConTinuum
                </div>
              </div>
              <div style={{ color: '#666', fontSize: '14px', textAlign: 'right' }}>
                <div>MdsLab - Università degli Studi di Messina</div>
                <div style={{ fontSize: '12px', color: '#999' }}>
                  Edge Device Management Platform
                </div>
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
                <Route path="/topology" element={<NetworkTopology />} />
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
