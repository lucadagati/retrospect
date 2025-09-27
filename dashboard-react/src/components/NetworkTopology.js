import React, { useState, useEffect } from 'react';
import { Card, Row, Col, Typography, Tag, Space, Tooltip, Badge } from 'antd';
import { 
  CloudServerOutlined, 
  GatewayOutlined, 
  DesktopOutlined,
  WifiOutlined,
  DisconnectOutlined,
  CheckCircleOutlined,
  ExclamationCircleOutlined
} from '@ant-design/icons';

const { Title, Text } = Typography;

const NetworkTopology = () => {
  const [topologyData, setTopologyData] = useState({
    gateways: [
      { id: 1, name: 'gateway-1', status: 'active', endpoint: '127.0.0.1:30452', devices: 2, region: 'us-west-1' },
      { id: 2, name: 'gateway-2', status: 'active', endpoint: '127.0.0.1:30454', devices: 2, region: 'us-west-1' },
      { id: 3, name: 'gateway-3', status: 'active', endpoint: '127.0.0.1:30456', devices: 2, region: 'us-west-1' }
    ],
    devices: [
      { id: 1, name: 'mcu-board-1', type: 'MCU', status: 'connected', gateway: 'gateway-1', architecture: 'riscv32' },
      { id: 2, name: 'mcu-board-2', type: 'MCU', status: 'connected', gateway: 'gateway-1', architecture: 'riscv32' },
      { id: 3, name: 'mcu-board-3', type: 'MCU', status: 'connected', gateway: 'gateway-2', architecture: 'riscv32' },
      { id: 4, name: 'riscv-board-1', type: 'RISC-V', status: 'connected', gateway: 'gateway-2', architecture: 'riscv64' },
      { id: 5, name: 'riscv-board-2', type: 'RISC-V', status: 'connected', gateway: 'gateway-3', architecture: 'riscv64' },
      { id: 6, name: 'riscv-board-3', type: 'RISC-V', status: 'connected', gateway: 'gateway-3', architecture: 'riscv64' }
    ],
    infrastructure: {
      status: 'active',
      endpoint: '127.0.0.1:30460',
      services: ['Certificate Authority', 'Secret Store', 'Monitoring']
    }
  });

  const [lastUpdate, setLastUpdate] = useState(new Date());

  useEffect(() => {
    const interval = setInterval(() => {
      setLastUpdate(new Date());
      // Simulate real-time updates
      setTopologyData(prev => ({
        ...prev,
        devices: prev.devices.map(device => ({
          ...device,
          status: Math.random() > 0.1 ? 'connected' : 'disconnected'
        }))
      }));
    }, 5000);

    return () => clearInterval(interval);
  }, []);

  const getStatusColor = (status) => {
    switch (status) {
      case 'active':
      case 'connected':
        return 'success';
      case 'inactive':
      case 'disconnected':
        return 'error';
      case 'pending':
        return 'warning';
      default:
        return 'default';
    }
  };

  const getStatusIcon = (status) => {
    switch (status) {
      case 'active':
      case 'connected':
        return <CheckCircleOutlined style={{ color: '#52c41a' }} />;
      case 'inactive':
      case 'disconnected':
        return <DisconnectOutlined style={{ color: '#ff4d4f' }} />;
      case 'pending':
        return <ExclamationCircleOutlined style={{ color: '#faad14' }} />;
      default:
        return <WifiOutlined style={{ color: '#d9d9d9' }} />;
    }
  };

  const getDeviceIcon = (type) => {
    switch (type) {
      case 'MCU':
        return <DesktopOutlined style={{ color: '#1890ff' }} />;
      case 'RISC-V':
        return <DesktopOutlined style={{ color: '#722ed1' }} />;
      default:
        return <DesktopOutlined style={{ color: '#666' }} />;
    }
  };

  return (
    <div>
      <div style={{ marginBottom: 24, display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
        <Title level={2}>Network Topology</Title>
        <div style={{ textAlign: 'right' }}>
          <Text type="secondary" style={{ fontSize: '12px' }}>
            Last updated: {lastUpdate.toLocaleTimeString()}
          </Text>
          <br />
          <Badge status="processing" text="Live Updates" />
        </div>
      </div>

      {/* Infrastructure Layer */}
      <Card 
        title={
          <Space>
            <CloudServerOutlined style={{ color: '#1890ff' }} />
            <span>Infrastructure Layer</span>
          </Space>
        }
        style={{ marginBottom: 16 }}
        size="small"
      >
        <Row gutter={[16, 16]}>
          <Col span={24}>
            <div style={{ 
              display: 'flex', 
              alignItems: 'center', 
              justifyContent: 'space-between',
              padding: '12px',
              background: '#f6ffed',
              borderRadius: '6px',
              border: '1px solid #b7eb8f'
            }}>
              <div>
                <Text strong>Infrastructure Service</Text>
                <br />
                <Text type="secondary" style={{ fontSize: '12px' }}>
                  {topologyData.infrastructure.endpoint}
                </Text>
              </div>
              <div style={{ textAlign: 'right' }}>
                <Tag color={getStatusColor(topologyData.infrastructure.status)}>
                  {getStatusIcon(topologyData.infrastructure.status)}
                  {topologyData.infrastructure.status.toUpperCase()}
                </Tag>
                <br />
                <Text type="secondary" style={{ fontSize: '11px' }}>
                  {topologyData.infrastructure.services.join(', ')}
                </Text>
              </div>
            </div>
          </Col>
        </Row>
      </Card>

      {/* Gateway Layer */}
      <Card 
        title={
          <Space>
            <GatewayOutlined style={{ color: '#52c41a' }} />
            <span>Gateway Layer</span>
          </Space>
        }
        style={{ marginBottom: 16 }}
        size="small"
      >
        <Row gutter={[16, 16]}>
          {topologyData.gateways.map(gateway => (
            <Col xs={24} sm={12} lg={8} key={gateway.id}>
              <div style={{ 
                display: 'flex', 
                alignItems: 'center', 
                justifyContent: 'space-between',
                padding: '12px',
                background: '#f6ffed',
                borderRadius: '6px',
                border: '1px solid #b7eb8f'
              }}>
                <div>
                  <Text strong>{gateway.name}</Text>
                  <br />
                  <Text type="secondary" style={{ fontSize: '12px' }}>
                    {gateway.endpoint}
                  </Text>
                  <br />
                  <Text type="secondary" style={{ fontSize: '11px' }}>
                    Region: {gateway.region}
                  </Text>
                </div>
                <div style={{ textAlign: 'right' }}>
                  <Tag color={getStatusColor(gateway.status)}>
                    {getStatusIcon(gateway.status)}
                    {gateway.status.toUpperCase()}
                  </Tag>
                  <br />
                  <Text type="secondary" style={{ fontSize: '11px' }}>
                    {gateway.devices} devices
                  </Text>
                </div>
              </div>
            </Col>
          ))}
        </Row>
      </Card>

      {/* Device Layer */}
      <Card 
        title={
          <Space>
            <DesktopOutlined style={{ color: '#722ed1' }} />
            <span>Device Layer</span>
          </Space>
        }
        size="small"
      >
        <Row gutter={[16, 16]}>
          {topologyData.devices.map(device => (
            <Col xs={24} sm={12} lg={8} xl={6} key={device.id}>
              <Tooltip title={`Architecture: ${device.architecture}`}>
                <div style={{ 
                  display: 'flex', 
                  alignItems: 'center', 
                  justifyContent: 'space-between',
                  padding: '12px',
                  background: device.status === 'connected' ? '#f6ffed' : '#fff2f0',
                  borderRadius: '6px',
                  border: `1px solid ${device.status === 'connected' ? '#b7eb8f' : '#ffccc7'}`
                }}>
                  <div>
                    <Space>
                      {getDeviceIcon(device.type)}
                      <Text strong>{device.name}</Text>
                    </Space>
                    <br />
                    <Text type="secondary" style={{ fontSize: '12px' }}>
                      {device.type} • {device.gateway}
                    </Text>
                  </div>
                  <div style={{ textAlign: 'right' }}>
                    <Tag color={getStatusColor(device.status)}>
                      {getStatusIcon(device.status)}
                      {device.status.toUpperCase()}
                    </Tag>
                  </div>
                </div>
              </Tooltip>
            </Col>
          ))}
        </Row>
      </Card>

      {/* Connection Summary */}
      <Card 
        title="Connection Summary"
        style={{ marginTop: 16 }}
        size="small"
      >
        <Row gutter={[16, 16]}>
          <Col xs={12} sm={6}>
            <div style={{ textAlign: 'center' }}>
              <div style={{ fontSize: '24px', fontWeight: 'bold', color: '#52c41a' }}>
                {topologyData.gateways.filter(g => g.status === 'active').length}
              </div>
              <Text type="secondary">Active Gateways</Text>
            </div>
          </Col>
          <Col xs={12} sm={6}>
            <div style={{ textAlign: 'center' }}>
              <div style={{ fontSize: '24px', fontWeight: 'bold', color: '#1890ff' }}>
                {topologyData.devices.length}
              </div>
              <Text type="secondary">Total Devices</Text>
            </div>
          </Col>
          <Col xs={12} sm={6}>
            <div style={{ textAlign: 'center' }}>
              <div style={{ fontSize: '24px', fontWeight: 'bold', color: '#52c41a' }}>
                {topologyData.devices.filter(d => d.status === 'connected').length}
              </div>
              <Text type="secondary">Connected Devices</Text>
            </div>
          </Col>
          <Col xs={12} sm={6}>
            <div style={{ textAlign: 'center' }}>
              <div style={{ fontSize: '24px', fontWeight: 'bold', color: '#faad14' }}>
                {topologyData.devices.filter(d => d.status === 'disconnected').length}
              </div>
              <Text type="secondary">Disconnected Devices</Text>
            </div>
          </Col>
        </Row>
      </Card>
    </div>
  );
};

export default NetworkTopology;
