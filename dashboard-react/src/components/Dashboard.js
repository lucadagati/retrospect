import React, { useState, useEffect } from 'react';
import { Card, Row, Col, Statistic, Spin, Alert, Typography } from 'antd';
import {
  DesktopOutlined,
  AppstoreOutlined,
  GatewayOutlined,
  CheckCircleOutlined,
  ExclamationCircleOutlined,
} from '@ant-design/icons';
import axios from 'axios';

const { Title } = Typography;

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
      const response = await axios.get('/api/status');
      setSystemStatus(response.data);
      setError(null);
    } catch (err) {
      setError('Failed to fetch system status');
      console.error('Error fetching system status:', err);
      // Set mock data for development
      setSystemStatus({
        devices: {
          total: 12,
          active: 8,
          inactive: 4,
          enrolling: 2
        },
        applications: {
          total: 5,
          running: 3,
          stopped: 1,
          failed: 1
        },
        gateways: {
          total: 2,
          active: 2,
          inactive: 0
        },
        infrastructure: {
          ca: 'active',
          monitoring: 'active',
          logging: 'active',
          secretStore: 'active'
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
