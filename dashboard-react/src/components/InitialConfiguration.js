import React, { useState, useEffect } from 'react';
import {
  Card,
  Steps,
  Button,
  Space,
  Typography,
  Row,
  Col,
  Alert,
  Progress,
  Spin,
  App,
  Divider,
  Tag,
  Tooltip,
} from 'antd';
import {
  SettingOutlined,
  GatewayOutlined,
  DesktopOutlined,
  CheckCircleOutlined,
  RocketOutlined,
  InfoCircleOutlined,
  CloudServerOutlined,
  NodeIndexOutlined,
} from '@ant-design/icons';

const { Title, Paragraph, Text } = Typography;
const { Step } = Steps;

const InitialConfiguration = () => {
  const { message } = App.useApp();
  const [currentStep, setCurrentStep] = useState(0);
  const [systemStatus, setSystemStatus] = useState({
    infrastructure: false,
    controllers: false,
    dashboard: false,
    gateways: 0,
    devices: 0,
  });
  const [loading, setLoading] = useState(false);
  const [configuring, setConfiguring] = useState(false);

  useEffect(() => {
    checkSystemStatus();
  }, []);

  const checkSystemStatus = async () => {
    setLoading(true);
    try {
      // Check infrastructure with shorter timeout
      let infraStatus = false;
      try {
        const controller = new AbortController();
        const timeoutId = setTimeout(() => controller.abort(), 3000); // 3 second timeout
        
        const infraResponse = await fetch('http://localhost:30461/health', {
          signal: controller.signal
        });
        clearTimeout(timeoutId);
        infraStatus = infraResponse.ok;
      } catch (e) {
        console.warn('Infrastructure not available:', e);
        infraStatus = false;
      }

      // Check controllers by testing API endpoints with shorter timeout
      let controllersStatus = false;
      try {
        const controller = new AbortController();
        const timeoutId = setTimeout(() => controller.abort(), 3000); // 3 second timeout
        
        const [devicesResponse, applicationsResponse, gatewaysResponse] = await Promise.all([
          fetch('/api/v1/devices', { signal: controller.signal }),
          fetch('/api/v1/applications', { signal: controller.signal }),
          fetch('/api/v1/gateways', { signal: controller.signal })
        ]);
        clearTimeout(timeoutId);
        controllersStatus = devicesResponse.ok && applicationsResponse.ok && gatewaysResponse.ok;
      } catch (e) {
        console.warn('Controllers not available:', e);
        controllersStatus = false;
      }

      // Check dashboard
      const dashboardStatus = true; // Dashboard is running

      // Check existing gateways and devices with shorter timeout
      let gateways = [];
      let devices = [];
      try {
        const controller = new AbortController();
        const timeoutId = setTimeout(() => controller.abort(), 3000); // 3 second timeout
        
        const [gatewaysResponse, devicesResponse] = await Promise.all([
          fetch('/api/v1/gateways', { signal: controller.signal }),
          fetch('/api/v1/devices', { signal: controller.signal })
        ]);
        clearTimeout(timeoutId);
        
        if (gatewaysResponse.ok) {
          const gatewaysData = await gatewaysResponse.json();
          gateways = gatewaysData.gateways || [];
        }
        
        if (devicesResponse.ok) {
          const devicesData = await devicesResponse.json();
          devices = devicesData.devices || [];
        }
      } catch (e) {
        console.warn('Failed to fetch gateways/devices:', e);
      }

      setSystemStatus({
        infrastructure: infraStatus,
        controllers: controllersStatus,
        dashboard: dashboardStatus,
        gateways: gateways.length,
        devices: devices.length,
      });

      // Auto-advance to next step if infrastructure is ready
      if (infraStatus && controllersStatus && dashboardStatus && currentStep === 0) {
        setCurrentStep(1);
      }
    } catch (error) {
      console.error('Error checking system status:', error);
      // Set default status on error
      setSystemStatus({
        infrastructure: false,
        controllers: false,
        dashboard: true,
        gateways: 0,
        devices: 0,
      });
    } finally {
      setLoading(false);
    }
  };

  const handleGatewayDeployment = async () => {
    setConfiguring(true);
    try {
      // Deploy default gateway configuration
      const response = await fetch('/api/v1/gateways', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          name: 'gateway-1',
          endpoint: '127.0.0.1:30452',
          tls_port: 30452,
          http_port: 30453,
          max_devices: 50,
          region: 'us-west-1',
          enabled: true,
        }),
      });

      if (response.ok) {
        message.success('Gateway deployed successfully!');
        await checkSystemStatus();
        setCurrentStep(2);
      } else {
        message.error('Failed to deploy gateway');
      }
    } catch (error) {
      console.error('Error deploying gateway:', error);
      message.error('Error deploying gateway');
    } finally {
      setConfiguring(false);
    }
  };

  const handleDeviceDeployment = async () => {
    setConfiguring(true);
    try {
      // Deploy sample devices
      const devices = [
        {
          name: 'esp32-board-1',
          type: 'ESP32',
          architecture: 'xtensa',
          gateway: 'gateway-1',
          enabled: true,
        },
        {
          name: 'stm32-board-1',
          type: 'STM32',
          architecture: 'arm',
          gateway: 'gateway-1',
          enabled: true,
        },
        {
          name: 'riscv-board-1',
          type: 'RISC-V',
          architecture: 'riscv64',
          gateway: 'gateway-1',
          enabled: true,
        },
      ];

      for (const device of devices) {
        const response = await fetch('/api/v1/devices', {
          method: 'POST',
          headers: {
            'Content-Type': 'application/json',
          },
          body: JSON.stringify(device),
        });

        if (!response.ok) {
          throw new Error(`Failed to deploy device ${device.name}`);
        }
      }

      message.success('Sample devices deployed successfully!');
      await checkSystemStatus();
      setCurrentStep(3);
    } catch (error) {
      console.error('Error deploying devices:', error);
      message.error('Error deploying devices');
    } finally {
      setConfiguring(false);
    }
  };

  const steps = [
    {
      title: 'System Check',
      icon: <CloudServerOutlined />,
      description: 'Verifying infrastructure and controllers',
      content: (
        <div>
          <Row gutter={[16, 16]}>
            <Col span={8}>
              <Card size="small" title="Infrastructure">
                <div style={{ textAlign: 'center' }}>
                  {systemStatus.infrastructure ? (
                    <CheckCircleOutlined style={{ fontSize: 24, color: '#52c41a' }} />
                  ) : (
                    <Spin size="small" />
                  )}
                  <div style={{ marginTop: 8 }}>
                    <Text type={systemStatus.infrastructure ? 'success' : 'secondary'}>
                      {systemStatus.infrastructure ? 'Running' : 'Checking...'}
                    </Text>
                  </div>
                </div>
              </Card>
            </Col>
            <Col span={8}>
              <Card size="small" title="Controllers">
                <div style={{ textAlign: 'center' }}>
                  {systemStatus.controllers ? (
                    <CheckCircleOutlined style={{ fontSize: 24, color: '#52c41a' }} />
                  ) : (
                    <Spin size="small" />
                  )}
                  <div style={{ marginTop: 8 }}>
                    <Text type={systemStatus.controllers ? 'success' : 'secondary'}>
                      {systemStatus.controllers ? 'Running' : 'Checking...'}
                    </Text>
                  </div>
                </div>
              </Card>
            </Col>
            <Col span={8}>
              <Card size="small" title="Dashboard">
                <div style={{ textAlign: 'center' }}>
                  {systemStatus.dashboard ? (
                    <CheckCircleOutlined style={{ fontSize: 24, color: '#52c41a' }} />
                  ) : (
                    <Spin size="small" />
                  )}
                  <div style={{ marginTop: 8 }}>
                    <Text type={systemStatus.dashboard ? 'success' : 'secondary'}>
                      {systemStatus.dashboard ? 'Running' : 'Checking...'}
                    </Text>
                  </div>
                </div>
              </Card>
            </Col>
          </Row>
          <Divider />
          <Alert
            message="System Status"
            description="All core services are running. You can now proceed to configure gateways and devices."
            type="success"
            showIcon
            style={{ marginTop: 16 }}
          />
        </div>
      ),
    },
    {
      title: 'Gateway Setup',
      icon: <GatewayOutlined />,
      description: 'Deploy and configure edge gateways',
      content: (
        <div>
          <Row gutter={[16, 16]}>
            <Col span={12}>
              <Card title="Gateway Configuration" size="small">
                <Paragraph>
                  Deploy your first gateway to handle device connections and application deployment.
                </Paragraph>
                <div style={{ marginBottom: 16 }}>
                  <Text strong>Default Configuration:</Text>
                  <ul style={{ marginTop: 8 }}>
                    <li>Name: gateway-1</li>
                    <li>Endpoint: 127.0.0.1:30452</li>
                    <li>Max Devices: 50</li>
                    <li>Region: us-west-1</li>
                  </ul>
                </div>
                <Button
                  type="primary"
                  icon={<RocketOutlined />}
                  onClick={handleGatewayDeployment}
                  loading={configuring}
                  disabled={systemStatus.gateways > 0}
                >
                  {systemStatus.gateways > 0 ? 'Gateway Already Deployed' : 'Deploy Gateway'}
                </Button>
              </Card>
            </Col>
            <Col span={12}>
              <Card title="Current Status" size="small">
                <div style={{ textAlign: 'center', padding: '20px 0' }}>
                  <div style={{ fontSize: 48, marginBottom: 16 }}>
                    {systemStatus.gateways > 0 ? (
                      <CheckCircleOutlined style={{ color: '#52c41a' }} />
                    ) : (
                      <GatewayOutlined style={{ color: '#1890ff' }} />
                    )}
                  </div>
                  <Title level={3} style={{ margin: 0 }}>
                    {systemStatus.gateways} Gateway{systemStatus.gateways !== 1 ? 's' : ''}
                  </Title>
                  <Text type="secondary">
                    {systemStatus.gateways > 0 ? 'Ready for devices' : 'Not deployed'}
                  </Text>
                </div>
              </Card>
            </Col>
          </Row>
        </div>
      ),
    },
    {
      title: 'Device Setup',
      icon: <DesktopOutlined />,
      description: 'Deploy sample devices for testing',
      content: (
        <div>
          <Row gutter={[16, 16]}>
            <Col span={12}>
              <Card title="Sample Devices" size="small">
                <Paragraph>
                  Deploy sample devices to test the platform functionality.
                </Paragraph>
                <div style={{ marginBottom: 16 }}>
                  <Text strong>Sample Devices:</Text>
                  <ul style={{ marginTop: 8 }}>
                    <li>ESP32 Board (Xtensa architecture)</li>
                    <li>STM32 Board (ARM architecture)</li>
                    <li>RISC-V Board (RISC-V architecture)</li>
                  </ul>
                </div>
                <Button
                  type="primary"
                  icon={<RocketOutlined />}
                  onClick={handleDeviceDeployment}
                  loading={configuring}
                  disabled={systemStatus.devices > 0}
                >
                  {systemStatus.devices > 0 ? 'Devices Already Deployed' : 'Deploy Sample Devices'}
                </Button>
              </Card>
            </Col>
            <Col span={12}>
              <Card title="Current Status" size="small">
                <div style={{ textAlign: 'center', padding: '20px 0' }}>
                  <div style={{ fontSize: 48, marginBottom: 16 }}>
                    {systemStatus.devices > 0 ? (
                      <CheckCircleOutlined style={{ color: '#52c41a' }} />
                    ) : (
                      <DesktopOutlined style={{ color: '#1890ff' }} />
                    )}
                  </div>
                  <Title level={3} style={{ margin: 0 }}>
                    {systemStatus.devices} Device{systemStatus.devices !== 1 ? 's' : ''}
                  </Title>
                  <Text type="secondary">
                    {systemStatus.devices > 0 ? 'Ready for applications' : 'Not deployed'}
                  </Text>
                </div>
              </Card>
            </Col>
          </Row>
        </div>
      ),
    },
    {
      title: 'Complete',
      icon: <CheckCircleOutlined />,
      description: 'Configuration completed successfully',
      content: (
        <div style={{ textAlign: 'center', padding: '40px 0' }}>
          <CheckCircleOutlined style={{ fontSize: 64, color: '#52c41a', marginBottom: 24 }} />
          <Title level={2}>Configuration Complete!</Title>
          <Paragraph style={{ fontSize: 16, marginBottom: 32 }}>
            Your Wasmbed platform is now ready for use. You can start deploying applications
            and managing your edge devices.
          </Paragraph>
          <Row gutter={[16, 16]} justify="center">
            <Col>
              <Tag color="blue" style={{ fontSize: 14, padding: '8px 16px' }}>
                <GatewayOutlined /> {systemStatus.gateways} Gateway{systemStatus.gateways !== 1 ? 's' : ''}
              </Tag>
            </Col>
            <Col>
              <Tag color="green" style={{ fontSize: 14, padding: '8px 16px' }}>
                <DesktopOutlined /> {systemStatus.devices} Device{systemStatus.devices !== 1 ? 's' : ''}
              </Tag>
            </Col>
          </Row>
          <div style={{ marginTop: 32 }}>
            <Button
              type="primary"
              size="large"
              onClick={() => window.location.href = '/dashboard'}
            >
              Go to Dashboard
            </Button>
          </div>
        </div>
      ),
    },
  ];

  return (
    <div style={{ padding: '24px', maxWidth: '1200px', margin: '0 auto' }}>
      <div style={{ textAlign: 'center', marginBottom: 32 }}>
        <Title level={1} style={{ marginBottom: 8 }}>
          <SettingOutlined /> Initial Configuration
        </Title>
        <Paragraph style={{ fontSize: 16, color: '#666' }}>
          Set up your Wasmbed platform by configuring gateways and devices
        </Paragraph>
      </div>

      <Card>
        <Steps current={currentStep} size="small" style={{ marginBottom: 32 }}>
          {steps.map((step, index) => (
            <Step
              key={index}
              title={step.title}
              description={step.description}
              icon={step.icon}
            />
          ))}
        </Steps>

        <div style={{ minHeight: '400px' }}>
          {steps[currentStep].content}
        </div>

        <div style={{ textAlign: 'center', marginTop: 32 }}>
          <Space>
            {currentStep > 0 && (
              <Button onClick={() => setCurrentStep(currentStep - 1)}>
                Previous
              </Button>
            )}
            {currentStep < steps.length - 1 && (
              <Button
                type="primary"
                onClick={() => setCurrentStep(currentStep + 1)}
                disabled={currentStep === 0 && (!systemStatus.infrastructure || !systemStatus.controllers)}
              >
                Next
              </Button>
            )}
            <Button
              icon={<InfoCircleOutlined />}
              onClick={checkSystemStatus}
              loading={loading}
            >
              Refresh Status
            </Button>
          </Space>
        </div>
      </Card>
    </div>
  );
};

export default InitialConfiguration;
