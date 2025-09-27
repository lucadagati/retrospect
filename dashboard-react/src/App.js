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
          colorPrimary: '#4a5568',
          borderRadius: 8,
          colorBgContainer: '#2d3748',
          colorBgElevated: '#4a5568',
          colorBgLayout: '#1a202c',
          colorBorder: '#4a5568',
          colorText: '#e2e8f0',
          colorTextSecondary: '#a0aec0',
          colorTextTertiary: '#718096',
          colorBgBase: '#1a202c',
          colorSuccess: '#68d391',
          colorWarning: '#f6e05e',
          colorError: '#fc8181',
          colorInfo: '#63b3ed',
          fontFamily: '-apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, "Helvetica Neue", Arial, sans-serif',
        },
        components: {
          Layout: {
            headerBg: '#2d3748',
            siderBg: '#1a202c',
            bodyBg: '#1a202c',
          },
          Menu: {
            itemBg: 'transparent',
            itemSelectedBg: '#4a5568',
            itemHoverBg: '#2d3748',
            darkItemBg: '#1a202c',
            darkItemSelectedBg: '#4a5568',
            darkItemHoverBg: '#2d3748',
            colorText: '#e2e8f0',
            colorTextSecondary: '#a0aec0',
          },
          Card: {
            borderRadius: 8,
            colorBgContainer: '#2d3748',
            colorBorder: '#4a5568',
            boxShadow: '0 1px 2px 0 rgba(0, 0, 0, 0.4), 0 1px 6px -1px rgba(0, 0, 0, 0.3), 0 2px 4px 0 rgba(0, 0, 0, 0.3)',
          },
          Table: {
            borderRadius: 8,
            headerBg: '#4a5568',
            colorBgContainer: '#2d3748',
            colorText: '#e2e8f0',
            colorTextHeading: '#e2e8f0',
            colorBorder: '#4a5568',
            colorSplit: '#4a5568',
          },
          Button: {
            colorBgContainer: '#4a5568',
            colorBorder: '#4a5568',
            colorText: '#e2e8f0',
            colorPrimary: '#4a5568',
            colorPrimaryHover: '#2d3748',
            colorPrimaryActive: '#1a202c',
          },
          Input: {
            colorBgContainer: '#4a5568',
            colorBorder: '#4a5568',
            colorText: '#e2e8f0',
            colorTextPlaceholder: '#a0aec0',
          },
          Select: {
            colorBgContainer: '#4a5568',
            colorBorder: '#4a5568',
            colorText: '#e2e8f0',
            colorTextPlaceholder: '#a0aec0',
          },
          Modal: {
            colorBgElevated: '#2d3748',
            colorText: '#e2e8f0',
          },
          Form: {
            labelColor: '#e2e8f0',
          },
          Typography: {
            colorText: '#e2e8f0',
            colorTextSecondary: '#a0aec0',
            colorTextTertiary: '#718096',
          },
          Tag: {
            colorText: '#e2e8f0',
          },
          Tooltip: {
            colorBgSpotlight: '#1a202c',
            colorTextLightSolid: '#e2e8f0',
          },
          Popconfirm: {
            colorBgElevated: '#2d3748',
            colorText: '#e2e8f0',
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
              height: 80, 
              margin: 16, 
              background: '#2d3748',
              borderRadius: borderRadiusLG,
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'center',
              flexDirection: 'column',
              padding: '12px',
              border: '1px solid #4a5568'
            }}>
              <img 
                src="/Logo_Serics.png" 
                alt="MdsLab Logo" 
                style={{ 
                  height: collapsed ? 32 : 48, 
                  width: 'auto',
                  marginBottom: collapsed ? 0 : 8
                }} 
              />
              {!collapsed && (
                <div style={{ 
                  fontSize: '12px', 
                  fontWeight: 'bold', 
                  color: '#e2e8f0',
                  textAlign: 'center',
                  lineHeight: 1.2
                }}>
                  RETROSPECT
                  <br />
                  <span style={{ fontSize: '9px', color: '#a0aec0' }}>
                    MdsLab - UniMe
                  </span>
                </div>
              )}
            </div>
            <Menu
              theme="dark"
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
              <div style={{ 
                color: '#a0aec0', 
                fontSize: '14px', 
                textAlign: 'right',
                display: 'flex',
                flexDirection: 'column',
                alignItems: 'flex-end',
                gap: '4px'
              }}>
                <div style={{ fontWeight: '500' }}>
                  MdsLab - Università degli Studi di Messina
                </div>
                <div style={{ 
                  fontSize: '12px', 
                  color: '#718096',
                  fontStyle: 'italic'
                }}>
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
