import React, { useState, useEffect } from 'react';
import { Card, Row, Col, Typography, Tag, Space, Tooltip, Badge } from 'antd';
import { 
  CloudServerOutlined, 
  GatewayOutlined, 
  DesktopOutlined,
  WifiOutlined,
  DisconnectOutlined,
  CheckCircleOutlined,
  ExclamationCircleOutlined,
  ArrowRightOutlined,
  ArrowDownOutlined
} from '@ant-design/icons';

const { Title, Text } = Typography;

const NetworkTopology = () => {
  const [topologyData, setTopologyData] = useState({
    gateways: [],
    devices: [],
    infrastructure: {
      status: 'unknown',
      endpoint: 'localhost:30460',
      services: ['Certificate Authority', 'Secret Store', 'Monitoring']
    }
  });

  const [lastUpdate, setLastUpdate] = useState(new Date());

  const fetchTopologyData = async () => {
    try {
      const [infrastructureResponse, gatewaysResponse, devicesResponse] = await Promise.all([
        fetch('/api/v1/infrastructure/status'),
        fetch('/api/v1/gateways'),
        fetch('/api/v1/devices')
      ]);

      const infrastructure = infrastructureResponse.ok ? await infrastructureResponse.json() : { status: 'unknown' };
      const gateways = gatewaysResponse.ok ? await gatewaysResponse.json() : { gateways: [] };
      const devices = devicesResponse.ok ? await devicesResponse.json() : { devices: [] };

      setTopologyData(prev => ({
        ...prev,
        infrastructure: {
          ...prev.infrastructure,
          status: infrastructure.status || 'unknown'
        },
        gateways: gateways.gateways || [],
        devices: devices.devices || []
      }));
    } catch (error) {
      console.error('Error fetching topology data:', error);
    }
  };

  useEffect(() => {
    fetchTopologyData();
    const interval = setInterval(() => {
      setLastUpdate(new Date());
      fetchTopologyData();
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
        return <DesktopOutlined style={{ color: '#63b3ed' }} />;
      case 'RISC-V':
        return <DesktopOutlined style={{ color: '#b794f6' }} />;
      default:
        return <DesktopOutlined style={{ color: '#a0aec0' }} />;
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
            <CloudServerOutlined style={{ color: '#63b3ed' }} />
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
              background: 'rgba(99, 179, 237, 0.1)',
              borderRadius: '6px',
              border: '1px solid rgba(99, 179, 237, 0.3)'
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
        
        {/* Connection Line */}
        <div style={{ 
          textAlign: 'center', 
          margin: '16px 0',
          height: '2px',
          background: 'linear-gradient(90deg, transparent 0%, #4a5568 20%, #4a5568 80%, transparent 100%)',
          position: 'relative'
        }}>
          <div style={{
            position: 'absolute',
            top: '-6px',
            left: '50%',
            transform: 'translateX(-50%)',
            width: '0',
            height: '0',
            borderLeft: '6px solid transparent',
            borderRight: '6px solid transparent',
            borderTop: '8px solid #4a5568'
          }} />
        </div>
      </Card>

      {/* Gateway Layer */}
      <Card 
        title={
          <Space>
            <GatewayOutlined style={{ color: '#68d391' }} />
            <span>Gateway Layer</span>
          </Space>
        }
        style={{ marginBottom: 16 }}
        size="small"
      >
        <Row gutter={[16, 16]}>
          {topologyData.gateways.map((gateway, index) => (
            <Col xs={24} sm={12} lg={8} key={gateway.id}>
              <div style={{ position: 'relative' }}>
                <div style={{ 
                  display: 'flex', 
                  alignItems: 'center', 
                  justifyContent: 'space-between',
                  padding: '12px',
                  background: 'rgba(104, 211, 145, 0.1)',
                  borderRadius: '6px',
                  border: '1px solid rgba(104, 211, 145, 0.3)'
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
                
                {/* Connection arrows to devices */}
                {gateway.devices > 0 && (
                  <div style={{
                    position: 'absolute',
                    bottom: '-6px', 
                    left: '50%', 
                    transform: 'translateX(-50%)',
                    zIndex: 10,
                    width: '0',
                    height: '0',
                    borderLeft: '4px solid transparent',
                    borderRight: '4px solid transparent',
                    borderTop: '6px solid #4a5568'
                  }} />
                )}
              </div>
            </Col>
          ))}
        </Row>
      </Card>

      {/* Device Layer */}
      <Card 
        title={
          <Space>
            <DesktopOutlined style={{ color: '#b794f6' }} />
            <span>Device Layer</span>
          </Space>
        }
        size="small"
      >
        <Row gutter={[16, 16]}>
          {topologyData.devices.map(device => (
            <Col xs={24} sm={12} lg={8} xl={6} key={device.id}>
              <Tooltip title={`Architecture: ${device.architecture} • Gateway: ${device.gateway}`}>
                <div style={{ 
                  display: 'flex', 
                  alignItems: 'center', 
                  justifyContent: 'space-between',
                  padding: '12px',
                  background: device.status === 'connected' 
                    ? 'rgba(104, 211, 145, 0.1)' 
                    : 'rgba(252, 129, 129, 0.1)',
                  borderRadius: '6px',
                  border: `1px solid ${device.status === 'connected' 
                    ? 'rgba(104, 211, 145, 0.3)' 
                    : 'rgba(252, 129, 129, 0.3)'}`,
                  position: 'relative'
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
                  
                  {/* Connection indicator */}
                  {device.status === 'connected' && (
                    <div style={{
                      position: 'absolute',
                      top: '8px', 
                      right: '8px',
                      width: '8px',
                      height: '8px',
                      borderRadius: '50%',
                      background: '#68d391'
                    }} />
                  )}
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
              <div style={{ fontSize: '24px', fontWeight: 'bold', color: '#68d391' }}>
                {topologyData.gateways.filter(g => g.status === 'active').length}
              </div>
              <Text type="secondary">Active Gateways</Text>
            </div>
          </Col>
          <Col xs={12} sm={6}>
            <div style={{ textAlign: 'center' }}>
              <div style={{ fontSize: '24px', fontWeight: 'bold', color: '#63b3ed' }}>
                {topologyData.devices.length}
              </div>
              <Text type="secondary">Total Devices</Text>
            </div>
          </Col>
          <Col xs={12} sm={6}>
            <div style={{ textAlign: 'center' }}>
              <div style={{ fontSize: '24px', fontWeight: 'bold', color: '#68d391' }}>
                {topologyData.devices.filter(d => d.status === 'connected').length}
              </div>
              <Text type="secondary">Connected Devices</Text>
            </div>
          </Col>
          <Col xs={12} sm={6}>
            <div style={{ textAlign: 'center' }}>
              <div style={{ fontSize: '24px', fontWeight: 'bold', color: '#f6e05e' }}>
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
