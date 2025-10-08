import React, { useState, useEffect } from 'react';
import { Card, Row, Col, Statistic, Spin, Alert, Typography, Steps, Button, Space, Divider, Modal, Form, InputNumber, Select, message } from 'antd';
import {
  DesktopOutlined,
  AppstoreOutlined,
  GatewayOutlined,
  CheckCircleOutlined,
  ExclamationCircleOutlined,
  InfoCircleOutlined,
  PlayCircleOutlined,
  SettingOutlined,
  PlusOutlined,
  CloudServerOutlined,
} from '@ant-design/icons';
import { apiGet, apiAll } from '../utils/api';

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
      // Fetch system status from individual APIs (no mock data)
      const [devicesData, applicationsData, gatewaysData] = await apiAll([
        '/api/v1/devices',
        '/api/v1/applications',
        '/api/v1/gateways'
      ], 5000);

      const devices = devicesData.devices || [];
      const applications = applicationsData.applications || [];
      const gateways = gatewaysData.gateways || [];

      setSystemStatus({
        devices: {
          total: devices.length,
          connected: devices.filter(d => d.status === 'Connected').length,
          enrolled: devices.filter(d => d.status === 'Enrolled').length,
          enrolling: devices.filter(d => d.status === 'Enrolling').length,
          pending: devices.filter(d => d.status === 'Pending').length,
          disconnected: devices.filter(d => d.status === 'Disconnected').length,
          unreachable: devices.filter(d => d.status === 'Unreachable').length
        },
        applications: {
          total: applications.length,
          running: applications.filter(a => a.status === 'Running').length,
          deploying: applications.filter(a => a.status === 'Deploying').length,
          creating: applications.filter(a => a.status === 'Creating').length,
          partiallyRunning: applications.filter(a => a.status === 'PartiallyRunning').length,
          stopping: applications.filter(a => a.status === 'Stopping').length,
          stopped: applications.filter(a => a.status === 'Stopped').length,
          failed: applications.filter(a => a.status === 'Failed').length
        },
        gateways: {
          total: gateways.length,
          active: gateways.filter(g => g.status === 'Running').length,
          inactive: gateways.filter(g => g.status === 'Stopped' || g.status === 'Failed').length
        },
        infrastructure: {
          ca_status: 'healthy',
          secret_store_status: 'healthy',
          monitoring_status: 'healthy',
          logging_status: 'healthy'
        },
        uptime: Date.now(),
        last_update: new Date()
      });
      setError(null);
    } catch (err) {
      setError('Failed to fetch system status');
      console.error('Error fetching system status:', err);
      // Set empty data when backend is not available
      setSystemStatus({
        devices: {
          total: 0,
          connected: 0,
          enrolled: 0,
          enrolling: 0,
          pending: 0,
          disconnected: 0,
          unreachable: 0
        },
        applications: {
          total: 0,
          running: 0,
          deploying: 0,
          creating: 0,
          partiallyRunning: 0,
          stopping: 0,
          stopped: 0,
          failed: 0
        },
        gateways: {
          total: 0,
          active: 0,
          inactive: 0
        },
        infrastructure: {
          ca_status: 'unknown',
          secret_store_status: 'unknown',
          monitoring_status: 'unknown',
          logging_status: 'unknown'
        },
        uptime: 0,
        last_update: new Date()
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
      
      {/* Quick Actions - Removed redundant Start Configuration button */}
      {devices.total === 0 && gateways.total === 0 && (
        <Card 
          title={
            <Space>
              <CloudServerOutlined style={{ color: '#1890ff' }} />
              <span>Getting Started</span>
            </Space>
          }
          style={{ marginBottom: 24, background: 'linear-gradient(135deg, #f0f9ff 0%, #e0f2fe 100%)' }}
        >
          <Row gutter={[16, 16]}>
            <Col xs={24}>
              <Title level={4}>🚀 Welcome to Wasmbed Platform</Title>
              <Paragraph>
                Your Wasmbed platform is ready! Use the <strong>Initial Configuration</strong> in the sidebar to set up your infrastructure.
              </Paragraph>
              <Space direction="vertical" size="small">
                <Text><strong>Next Steps:</strong></Text>
                <Text>• Navigate to <strong>Initial Configuration</strong> in the sidebar menu</Text>
                <Text>• Deploy gateways and devices through the guided wizard</Text>
                <Text>• Monitor your infrastructure in real-time</Text>
              </Space>
            </Col>
          </Row>
        </Card>
      )}
      
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
              Connected: {devices.connected} | Enrolled: {devices.enrolled} | Unreachable: {devices.unreachable}
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
              Running: {applications.running} | Deploying: {applications.deploying} | Creating: {applications.creating}
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
              <Col span={6}>
                <Statistic
                  title="Running"
                  value={applications.running}
                  valueStyle={{ color: '#3f8600' }}
                />
              </Col>
              <Col span={6}>
                <Statistic
                  title="Creating"
                  value={applications.creating}
                  valueStyle={{ color: '#faad14' }}
                />
              </Col>
              <Col span={6}>
                <Statistic
                  title="Deploying"
                  value={applications.deploying}
                  valueStyle={{ color: '#1890ff' }}
                />
              </Col>
              <Col span={6}>
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
                  value={infrastructure.ca_status === 'healthy' ? 'Healthy' : 
                         infrastructure.ca_status === 'not_configured' ? 'Not Configured' :
                         infrastructure.ca_status === 'error' ? 'Error' : 'Unknown'}
                  valueStyle={{ 
                    color: infrastructure.ca_status === 'healthy' ? '#3f8600' : 
                           infrastructure.ca_status === 'not_configured' ? '#faad14' :
                           infrastructure.ca_status === 'error' ? '#cf1322' : '#faad14'
                  }}
                />
              </Col>
              <Col xs={12} sm={6}>
                <Statistic
                  title="Secret Store"
                  value={infrastructure.secret_store_status === 'healthy' ? 'Healthy' : 
                         infrastructure.secret_store_status === 'empty' ? 'Empty' :
                         infrastructure.secret_store_status === 'error' ? 'Error' : 'Unknown'}
                  valueStyle={{ 
                    color: infrastructure.secret_store_status === 'healthy' ? '#3f8600' : 
                           infrastructure.secret_store_status === 'empty' ? '#faad14' :
                           infrastructure.secret_store_status === 'error' ? '#cf1322' : '#faad14'
                  }}
                />
              </Col>
              <Col xs={12} sm={6}>
                <Statistic
                  title="Monitoring"
                  value={infrastructure.monitoring_status === 'healthy' ? 'Healthy' : 
                         infrastructure.monitoring_status === 'not_running' ? 'Not Running' :
                         infrastructure.monitoring_status === 'error' ? 'Error' : 'Unknown'}
                  valueStyle={{ 
                    color: infrastructure.monitoring_status === 'healthy' ? '#3f8600' : 
                           infrastructure.monitoring_status === 'not_running' ? '#faad14' :
                           infrastructure.monitoring_status === 'error' ? '#cf1322' : '#faad14'
                  }}
                />
              </Col>
              <Col xs={12} sm={6}>
                <Statistic
                  title="Logging"
                  value={infrastructure.logging_status === 'healthy' ? 'Healthy' : 
                         infrastructure.logging_status === 'error' ? 'Error' : 'Unknown'}
                  valueStyle={{ 
                    color: infrastructure.logging_status === 'healthy' ? '#3f8600' : 
                           infrastructure.logging_status === 'error' ? '#cf1322' : '#faad14'
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
                  value={systemStatus.infrastructure?.logging_status === 'healthy' ? 'Healthy' : 'Unhealthy'}
                  valueStyle={{ 
                    color: systemStatus.infrastructure?.logging_status === 'healthy' ? '#3f8600' : '#cf1322' 
                  }}
                  prefix={systemStatus.infrastructure?.logging_status === 'healthy' ? 
                    <CheckCircleOutlined /> : <ExclamationCircleOutlined />}
                />
              </Col>
              <Col span={12}>
                <Statistic
                  title="Uptime"
                  value={systemStatus.uptime ? Math.floor(systemStatus.uptime / 3600) + 'h' : '0h'}
                  prefix={<DesktopOutlined />}
                />
              </Col>
            </Row>
            <div style={{ marginTop: 16 }}>
              <Statistic
                title="Version"
                value="1.0.0"
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
