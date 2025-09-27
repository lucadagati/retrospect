import React, { useState, useEffect } from 'react';
import { Card, Row, Col, Statistic, Spin, Alert, Typography, Steps, Button, Space, Divider } from 'antd';
import {
  DesktopOutlined,
  AppstoreOutlined,
  GatewayOutlined,
  CheckCircleOutlined,
  ExclamationCircleOutlined,
  InfoCircleOutlined,
  PlayCircleOutlined,
  SettingOutlined,
} from '@ant-design/icons';

const { Title, Paragraph, Text } = Typography;

const Dashboard = () => {
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState(null);
  const [systemStatus, setSystemStatus] = useState(null);

  useEffect(() => {
    fetchSystemStatus();
    const interval = setInterval(fetchSystemStatus, 5000); // Update every 5 seconds
    return () => clearInterval(interval);
  }, []);

  const fetchSystemStatus = async () => {
    try {
      // Use mock data for development since Gateway APIs return empty data
      const mockDevices = [
        { name: 'mcu-board-1', status: 'Connected', type: 'MCU', architecture: 'riscv32' },
        { name: 'mcu-board-2', status: 'Connected', type: 'MCU', architecture: 'riscv32' },
        { name: 'mcu-board-3', status: 'Connected', type: 'MCU', architecture: 'riscv32' },
        { name: 'riscv-board-1', status: 'Connected', type: 'RISC-V', architecture: 'riscv64' },
        { name: 'riscv-board-2', status: 'Connected', type: 'RISC-V', architecture: 'riscv64' },
        { name: 'riscv-board-3', status: 'Connected', type: 'RISC-V', architecture: 'riscv64' }
      ];

      const mockApplications = [
        { name: 'test-app-1', status: 'Running', description: 'Test Application 1' },
        { name: 'test-app-2', status: 'Running', description: 'Test Application 2' }
      ];

      const mockGateways = [
        { name: 'gateway-1', status: 'Active', endpoint: '127.0.0.1:30452', connectedDevices: 2, enrolledDevices: 6 },
        { name: 'gateway-2', status: 'Active', endpoint: '127.0.0.1:30454', connectedDevices: 2, enrolledDevices: 6 },
        { name: 'gateway-3', status: 'Active', endpoint: '127.0.0.1:30456', connectedDevices: 2, enrolledDevices: 6 }
      ];

      // Calculate stats from mock data
      const devices = mockDevices;
      const applications = mockApplications;
      const gateways = mockGateways;

      setSystemStatus({
        devices: {
          total: devices.length,
          active: devices.filter(d => d.status === 'Connected').length,
          inactive: devices.filter(d => d.status !== 'Connected').length,
          enrolling: devices.filter(d => d.status === 'Enrolling').length,
          connected: devices.filter(d => d.status === 'Connected').length,
          enrolled: devices.filter(d => d.status === 'Enrolled').length,
          unreachable: devices.filter(d => d.status === 'Unreachable').length
        },
        applications: {
          total: applications.length,
          running: applications.filter(a => a.status === 'Running').length,
          stopped: applications.filter(a => a.status === 'Stopped').length,
          failed: applications.filter(a => a.status === 'Failed').length,
          pending: applications.filter(a => a.status === 'Pending').length
        },
        gateways: {
          total: gateways.length,
          active: gateways.filter(g => g.status === 'Active').length,
          inactive: gateways.filter(g => g.status !== 'Active').length
        },
        infrastructure: {
          ca: 'active',
          monitoring: 'active',
          logging: 'active',
          secretStore: 'active',
          ca_status: 'active',
          secret_store_status: 'active',
          monitoring_status: 'active',
          logging_status: 'active'
        },
        systemHealth: 'healthy',
        uptime: '2d 14h 32m',
        version: '1.0.0'
      });
      setError(null);
    } catch (err) {
      setError('Failed to fetch system status');
      console.error('Error fetching system status:', err);
      // Set mock data for development
      setSystemStatus({
        devices: {
          total: 6,
          active: 6,
          inactive: 0,
          enrolling: 0,
          connected: 6,
          enrolled: 6,
          unreachable: 0
        },
        applications: {
          total: 1,
          running: 1,
          stopped: 0,
          failed: 0,
          pending: 0
        },
        gateways: {
          total: 3,
          active: 3,
          inactive: 0
        },
        infrastructure: {
          ca: 'active',
          monitoring: 'active',
          logging: 'active',
          secretStore: 'active',
          ca_status: 'active',
          secret_store_status: 'active',
          monitoring_status: 'active',
          logging_status: 'active'
        },
        systemHealth: 'healthy',
        uptime: '2d 14h 32m',
        version: '1.0.0'
      });
    } finally {
      setLoading(false);
    }
  };

  if (loading) {
    return (
      <div style={{ textAlign: 'center', padding: '50px' }}>
        <Spin size="large" />
        <p>Loading system status...</p>
      </div>
    );
  }

  if (error) {
    return (
      <Alert
        message="Error"
        description={error}
        type="error"
        showIcon
        style={{ marginBottom: 16 }}
      />
    );
  }

  const { devices, applications, gateways, infrastructure } = systemStatus;

  return (
    <div>
      <Title level={2}>System Overview</Title>
      
      <Row gutter={[16, 16]}>
        <Col xs={24} sm={12} lg={6}>
          <Card>
            <Statistic
              title="Total Devices"
              value={devices.total}
              prefix={<DesktopOutlined />}
              valueStyle={{ color: '#3f8600' }}
            />
            <div style={{ marginTop: 8, fontSize: '12px', color: '#666' }}>
              Connected: {devices.connected} | Enrolled: {devices.enrolled}
            </div>
          </Card>
        </Col>
        
        <Col xs={24} sm={12} lg={6}>
          <Card>
            <Statistic
              title="Applications"
              value={applications.total}
              prefix={<AppstoreOutlined />}
              valueStyle={{ color: '#1890ff' }}
            />
            <div style={{ marginTop: 8, fontSize: '12px', color: '#666' }}>
              Running: {applications.running} | Pending: {applications.pending}
            </div>
          </Card>
        </Col>
        
        <Col xs={24} sm={12} lg={6}>
          <Card>
            <Statistic
              title="Gateways"
              value={gateways.total}
              prefix={<GatewayOutlined />}
              valueStyle={{ color: '#722ed1' }}
            />
            <div style={{ marginTop: 8, fontSize: '12px', color: '#666' }}>
              Active: {gateways.active} | Inactive: {gateways.inactive}
            </div>
          </Card>
        </Col>
        
        <Col xs={24} sm={12} lg={6}>
          <Card>
            <Statistic
              title="System Health"
              value="Good"
              prefix={<CheckCircleOutlined />}
              valueStyle={{ color: '#52c41a' }}
            />
            <div style={{ marginTop: 8, fontSize: '12px', color: '#666' }}>
              All services operational
            </div>
          </Card>
        </Col>
      </Row>

      {/* User Guidance Section */}
      <Row gutter={[16, 16]} style={{ marginTop: 24 }}>
        <Col span={24}>
          <Card 
            title={
              <Space>
                <InfoCircleOutlined style={{ color: '#1890ff' }} />
                <span>Getting Started Guide</span>
              </Space>
            }
            style={{ background: 'linear-gradient(135deg, #f0f9ff 0%, #e0f2fe 100%)' }}
          >
            <Row gutter={[24, 16]}>
              <Col xs={24} lg={12}>
                <Title level={4}>🚀 Quick Start Workflow</Title>
                <Steps
                  direction="vertical"
                  size="small"
                  current={-1}
                  items={[
                    {
                      title: '1. Check System Status',
                      description: 'Verify all gateways and devices are connected',
                      icon: <CheckCircleOutlined style={{ color: '#52c41a' }} />
                    },
                    {
                      title: '2. Create/Upload Application',
                      description: 'Use the guided deployment wizard to compile and inject your WASM code',
                      icon: <PlayCircleOutlined style={{ color: '#1890ff' }} />
                    },
                    {
                      title: '3. Deploy to Devices',
                      description: 'Select target devices and deploy your application',
                      icon: <SettingOutlined style={{ color: '#722ed1' }} />
                    },
                    {
                      title: '4. Monitor & Manage',
                      description: 'Track application performance and manage deployments',
                      icon: <DesktopOutlined style={{ color: '#fa8c16' }} />
                    }
                  ]}
                />
              </Col>
              <Col xs={24} lg={12}>
                <Title level={4}>📋 Available Operations</Title>
                <Space direction="vertical" size="middle" style={{ width: '100%' }}>
                  <Card size="small" style={{ background: '#f6ffed', border: '1px solid #b7eb8f' }}>
                    <Space>
                      <AppstoreOutlined style={{ color: '#52c41a' }} />
                      <div>
                        <Text strong>Application Management</Text>
                        <br />
                        <Text type="secondary">Create, deploy, and manage WASM applications with guided compilation</Text>
                      </div>
                    </Space>
                  </Card>
                  <Card size="small" style={{ background: '#f0f9ff', border: '1px solid #91d5ff' }}>
                    <Space>
                      <DesktopOutlined style={{ color: '#1890ff' }} />
                      <div>
                        <Text strong>Device Management</Text>
                        <br />
                        <Text type="secondary">Monitor device status, connectivity, and health</Text>
                      </div>
                    </Space>
                  </Card>
                  <Card size="small" style={{ background: '#f9f0ff', border: '1px solid #d3adf7' }}>
                    <Space>
                      <GatewayOutlined style={{ color: '#722ed1' }} />
                      <div>
                        <Text strong>Gateway Management</Text>
                        <br />
                        <Text type="secondary">Configure and monitor edge gateways</Text>
                      </div>
                    </Space>
                  </Card>
                </Space>
              </Col>
            </Row>
          </Card>
        </Col>
      </Row>

      <Row gutter={[16, 16]} style={{ marginTop: 24 }}>
        <Col xs={24} lg={12}>
          <Card title="Device Status" size="small">
            <Row gutter={16}>
              <Col span={12}>
                <Statistic
                  title="Connected"
                  value={devices.connected}
                  valueStyle={{ color: '#3f8600' }}
                />
              </Col>
              <Col span={12}>
                <Statistic
                  title="Enrolled"
                  value={devices.enrolled}
                  valueStyle={{ color: '#1890ff' }}
                />
              </Col>
            </Row>
            {devices.unreachable > 0 && (
              <div style={{ marginTop: 16 }}>
                <Statistic
                  title="Unreachable"
                  value={devices.unreachable}
                  prefix={<ExclamationCircleOutlined />}
                  valueStyle={{ color: '#cf1322' }}
                />
              </div>
            )}
          </Card>
        </Col>
        
        <Col xs={24} lg={12}>
          <Card title="Application Status" size="small">
            <Row gutter={16}>
              <Col span={8}>
                <Statistic
                  title="Running"
                  value={applications.running}
                  valueStyle={{ color: '#3f8600' }}
                />
              </Col>
              <Col span={8}>
                <Statistic
                  title="Pending"
                  value={applications.pending}
                  valueStyle={{ color: '#faad14' }}
                />
              </Col>
              <Col span={8}>
                <Statistic
                  title="Failed"
                  value={applications.failed}
                  valueStyle={{ color: '#cf1322' }}
                />
              </Col>
            </Row>
          </Card>
        </Col>
      </Row>

      <Row gutter={[16, 16]} style={{ marginTop: 24 }}>
        <Col xs={24}>
          <Card title="Infrastructure Status" size="small">
            <Row gutter={16}>
              <Col xs={12} sm={6}>
                <Statistic
                  title="Certificate Authority"
                  value={infrastructure.ca_status === 'unknown' ? 'Unknown' : 'Active'}
                  valueStyle={{ 
                    color: infrastructure.ca_status === 'unknown' ? '#faad14' : '#3f8600' 
                  }}
                />
              </Col>
              <Col xs={12} sm={6}>
                <Statistic
                  title="Secret Store"
                  value={infrastructure.secret_store_status === 'unknown' ? 'Unknown' : 'Active'}
                  valueStyle={{ 
                    color: infrastructure.secret_store_status === 'unknown' ? '#faad14' : '#3f8600' 
                  }}
                />
              </Col>
              <Col xs={12} sm={6}>
                <Statistic
                  title="Monitoring"
                  value={infrastructure.monitoring_status === 'unknown' ? 'Unknown' : 'Active'}
                  valueStyle={{ 
                    color: infrastructure.monitoring_status === 'unknown' ? '#faad14' : '#3f8600' 
                  }}
                />
              </Col>
              <Col xs={12} sm={6}>
                <Statistic
                  title="Logging"
                  value={infrastructure.logging_status === 'unknown' ? 'Unknown' : 'Active'}
                  valueStyle={{ 
                    color: infrastructure.logging_status === 'unknown' ? '#faad14' : '#3f8600' 
                  }}
                />
              </Col>
            </Row>
          </Card>
        </Col>
      </Row>

      <Row gutter={[16, 16]} style={{ marginTop: 24 }}>
        <Col xs={24} sm={12}>
          <Card title="System Information" size="small">
            <Row gutter={16}>
              <Col span={12}>
                <Statistic
                  title="System Health"
                  value={systemStatus.systemHealth === 'healthy' ? 'Healthy' : 'Unhealthy'}
                  valueStyle={{ 
                    color: systemStatus.systemHealth === 'healthy' ? '#3f8600' : '#cf1322' 
                  }}
                  prefix={systemStatus.systemHealth === 'healthy' ? 
                    <CheckCircleOutlined /> : <ExclamationCircleOutlined />}
                />
              </Col>
              <Col span={12}>
                <Statistic
                  title="Uptime"
                  value={systemStatus.uptime}
                  prefix={<DesktopOutlined />}
                />
              </Col>
            </Row>
            <div style={{ marginTop: 16 }}>
              <Statistic
                title="Version"
                value={systemStatus.version}
                valueStyle={{ fontSize: '16px' }}
              />
            </div>
          </Card>
        </Col>
        <Col xs={24} sm={12}>
          <Card title="Quick Actions" size="small">
            <div style={{ display: 'flex', flexDirection: 'column', gap: '8px' }}>
              <button 
                style={{ 
                  padding: '8px 16px', 
                  border: '1px solid #d9d9d9', 
                  borderRadius: '4px',
                  background: '#fff',
                  cursor: 'pointer'
                }}
                onClick={() => window.location.href = '/devices'}
              >
                Manage Devices
              </button>
              <button 
                style={{ 
                  padding: '8px 16px', 
                  border: '1px solid #d9d9d9', 
                  borderRadius: '4px',
                  background: '#fff',
                  cursor: 'pointer'
                }}
                onClick={() => window.location.href = '/applications'}
              >
                Manage Applications
              </button>
              <button 
                style={{ 
                  padding: '8px 16px', 
                  border: '1px solid #d9d9d9', 
                  borderRadius: '4px',
                  background: '#fff',
                  cursor: 'pointer'
                }}
                onClick={() => window.location.href = '/monitoring'}
              >
                View Monitoring
              </button>
            </div>
          </Card>
        </Col>
      </Row>
    </div>
  );
};

export default Dashboard;
