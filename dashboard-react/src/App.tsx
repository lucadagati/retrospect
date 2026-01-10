import React, { useState, useEffect } from 'react';
import {
  Layout,
  Menu,
  Card,
  Row,
  Col,
  Statistic,
  Table,
  Button,
  Space,
  Typography,
  Progress,
  Tag,
  message,
  Spin
} from 'antd';
import {
  DashboardOutlined,
  CloudServerOutlined,
  MobileOutlined,
  AppstoreOutlined,
  SettingOutlined,
  PlayCircleOutlined,
  PauseCircleOutlined,
  ReloadOutlined,
  MonitorOutlined,
  ConsoleSqlOutlined,
  NodeIndexOutlined,
  RocketOutlined
} from '@ant-design/icons';
import './App.css';

// Import all components
import Dashboard from './components/Dashboard';
import DeviceManagement from './components/DeviceManagement';
import ApplicationManagement from './components/ApplicationManagement';
import GatewayManagement from './components/GatewayManagement';
import Monitoring from './components/Monitoring';
import Terminal from './components/Terminal';
import NetworkTopology from './components/NetworkTopology';
import InitialConfiguration from './components/InitialConfiguration';

const { Header, Content, Sider } = Layout;
const { Title, Text } = Typography;

interface Device {
  id: number;
  name: string;
  status: string;
  type: string;
  architecture: string;
  mcuType?: string;
  lastHeartbeat: string;
  renodeInstance?: string;
  emulationStatus?: string;
}

interface Application {
  id: number;
  name: string;
  status: string;
  targetDevices: string[];
  wasmBytes: string;
}

interface Gateway {
  id: number;
  name: string;
  status: string;
  endpoint: string;
  connectedDevices: number;
  maxDevices: number;
}

const App: React.FC = () => {
  const [loading, setLoading] = useState(false);
  const [devices, setDevices] = useState<Device[]>([]);
  const [applications, setApplications] = useState<Application[]>([]);
  const [gateways, setGateways] = useState<Gateway[]>([]);
  const [selectedKey, setSelectedKey] = useState('dashboard');

  // No mock data - use real APIs only

  const fetchData = async () => {
    setLoading(true);
    try {
      // Fetch real data from APIs with proper error handling
      const [devicesResponse, applicationsResponse, gatewaysResponse] = await Promise.all([
        fetch('/api/v1/devices').catch(err => {
          console.warn('Devices API failed:', err);
          return { json: () => Promise.resolve({ devices: [] }) };
        }),
        fetch('/api/v1/applications').catch(err => {
          console.warn('Applications API failed:', err);
          return { json: () => Promise.resolve({ applications: [] }) };
        }),
        fetch('/api/v1/gateways').catch(err => {
          console.warn('Gateways API failed:', err);
          return { json: () => Promise.resolve({ gateways: [] }) };
        })
      ]);

      const devicesData = await devicesResponse.json();
      const applicationsData = await applicationsResponse.json();
      const gatewaysData = await gatewaysResponse.json();

      // Transform data to match interface expectations
      const transformedDevices = (devicesData.devices || []).map((device: any) => ({
        id: device.device_id,
        name: device.device_id,
        status: device.status,
        type: device.device_type || 'MCU', // Default to MCU
        architecture: 'ARM_CORTEX_M', // Fixed architecture for Renode emulation
        mcuType: device.mcu_type || 'RenodeArduinoNano33Ble', // Default MCU type
        lastHeartbeat: device.last_heartbeat ? new Date(device.last_heartbeat.secs_since_epoch * 1000).toISOString() : null,
        renodeInstance: device.renode_instance || null,
        emulationStatus: device.emulation_status || 'Not Started'
      }));

      const transformedApplications = (applicationsData.applications || []).map((app: any) => ({
        id: app.app_id,
        name: app.name,
        status: app.status,
        targetDevices: app.targetDevices?.deviceNames || [],
        wasmBytes: app.wasmBytes || 'N/A'
      }));

      const transformedGateways = (gatewaysData.gateways || []).map((gateway: any) => ({
        id: gateway.gateway_id || gateway.name,
        name: gateway.name,
        status: gateway.status,
        endpoint: gateway.endpoint,
        connectedDevices: gateway.connectedDevices || 0,
        maxDevices: gateway.maxDevices || 100
      }));

      setDevices(transformedDevices);
      setApplications(transformedApplications);
      setGateways(transformedGateways);
    } catch (error) {
      console.error('Error fetching data:', error);
      // Set empty arrays on error
      setDevices([]);
      setApplications([]);
      setGateways([]);
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    fetchData();
    const interval = setInterval(fetchData, 30000); // Refresh every 30 seconds
    return () => clearInterval(interval);
  }, []);

  const deviceColumns = [
    {
      title: 'Name',
      dataIndex: 'name',
      key: 'name',
    },
    {
      title: 'Status',
      dataIndex: 'status',
      key: 'status',
      render: (status: string) => {
        const color = status === 'Connected' ? 'green' : 
                     status === 'Enrolled' ? 'blue' : 
                     status === 'Enrolling' ? 'cyan' : 
                     status === 'Pending' ? 'orange' : 
                     status === 'Disconnected' ? 'gray' : 'red';
        return (
          <Tag color={color}>
            {status}
          </Tag>
        );
      },
    },
    {
      title: 'Type',
      dataIndex: 'type',
      key: 'type',
    },
        {
          title: 'Architecture',
          dataIndex: 'architecture',
          key: 'architecture',
          render: (arch: string) => (
            <Tag color="blue">ARM Cortex-M (Renode)</Tag>
          ),
        },
        {
          title: 'MCU Type',
          dataIndex: 'mcuType',
          key: 'mcuType',
          render: (mcuType: string) => {
            const mcuNames: { [key: string]: string } = {
              // Renode-supported boards
              'RenodeArduinoNano33Ble': 'Arduino Nano 33 BLE (Cortex-M4)',
              'RenodeStm32F4Discovery': 'STM32F4 Discovery (Cortex-M4)',
              // Legacy MPS2 boards (mapped to Renode boards)
              'Mps2An385': 'ARM MPS2-AN385 (Cortex-M3) - Mapped to Arduino Nano',
              'Mps2An386': 'ARM MPS2-AN386 (Cortex-M4) - Legacy',
              'Mps2An500': 'ARM MPS2-AN500 (Cortex-M7) - Legacy',
              'Mps2An505': 'ARM MPS2-AN505 (Cortex-M33) - Legacy',
              // Legacy STM32 boards
              'Stm32Vldiscovery': 'STM32VLDISCOVERY (Cortex-M3) - Legacy',
              'OlimexStm32H405': 'Olimex STM32-H405 (Cortex-M4) - Legacy',
            };
            return (
              <Tag color="green">
                {mcuNames[mcuType] || mcuType}
              </Tag>
            );
          },
        },
    {
      title: 'Renode Status',
      dataIndex: 'emulationStatus',
      key: 'emulationStatus',
      render: (status: string) => {
        const color = status === 'Running' ? 'green' : 
                     status === 'Starting' ? 'orange' : 
                     status === 'Stopped' ? 'red' : 'default';
        return (
          <Tag color={color}>
            {status}
          </Tag>
        );
      },
    },
    {
      title: 'Last Heartbeat',
      dataIndex: 'lastHeartbeat',
      key: 'lastHeartbeat',
      render: (time: string) => time ? new Date(time).toLocaleString() : 'Never',
    },
  ];

  const applicationColumns = [
    {
      title: 'Name',
      dataIndex: 'name',
      key: 'name',
    },
    {
      title: 'Status',
      dataIndex: 'status',
      key: 'status',
      render: (status: string) => {
        const color = status === 'Running' ? 'green' : 
                     status === 'Deploying' ? 'blue' : 
                     status === 'Creating' ? 'orange' : 
                     status === 'PartiallyRunning' ? 'cyan' : 
                     status === 'Stopping' ? 'purple' : 
                     status === 'Stopped' ? 'gray' : 
                     status === 'Failed' ? 'red' : 'default';
        return (
          <Tag color={color}>
            {status}
          </Tag>
        );
      },
    },
    {
      title: 'Target Devices',
      dataIndex: 'targetDevices',
      key: 'targetDevices',
      render: (devices: string[]) => devices.length > 0 ? devices.join(', ') : 'None',
    },
    {
      title: 'WASM Size',
      dataIndex: 'wasmBytes',
      key: 'wasmBytes',
      render: (bytes: string) => bytes === 'N/A' ? 'N/A' : `${bytes.length} bytes`,
    },
  ];

  const gatewayColumns = [
    {
      title: 'Name',
      dataIndex: 'name',
      key: 'name',
    },
    {
      title: 'Status',
      dataIndex: 'status',
      key: 'status',
      render: (status: string) => {
        const color = status === 'Running' ? 'green' : 
                     status === 'Pending' ? 'orange' : 
                     status === 'Stopped' ? 'gray' : 'red';
        return (
          <Tag color={color}>
            {status}
          </Tag>
        );
      },
    },
    {
      title: 'Endpoint',
      dataIndex: 'endpoint',
      key: 'endpoint',
    },
    {
      title: 'Connected Devices',
      dataIndex: 'connectedDevices',
      key: 'connectedDevices',
      render: (connected: number, record: Gateway) => (
        <Progress
          percent={(connected / record.maxDevices) * 100}
          format={() => `${connected}/${record.maxDevices}`}
          size="small"
        />
      ),
    },
  ];

  const renderDashboard = () => (
    <div>
      <Title level={2}>Wasmbed Platform Dashboard</Title>
      
      <Row gutter={16} style={{ marginBottom: 24 }}>
        <Col span={6}>
          <Card>
            <Statistic
              title="Total Devices"
              value={devices.length}
              prefix={<MobileOutlined />}
              valueStyle={{ color: '#3f8600' }}
            />
          </Card>
        </Col>
        <Col span={6}>
          <Card>
            <Statistic
              title="Active Applications"
              value={applications.filter(app => app.status === 'Running' || app.status === 'Deploying').length}
              prefix={<AppstoreOutlined />}
              valueStyle={{ color: '#3f8600' }}
            />
          </Card>
        </Col>
        <Col span={6}>
          <Card>
            <Statistic
              title="Active Gateways"
              value={gateways.filter(gw => gw.status === 'Running').length}
              prefix={<CloudServerOutlined />}
              valueStyle={{ color: '#3f8600' }}
            />
          </Card>
        </Col>
        <Col span={6}>
          <Card>
            <Statistic
              title="Enrolled Devices"
              value={devices.filter(device => device.status === 'Enrolled' || device.status === 'Connected').length}
              prefix={<MobileOutlined />}
              valueStyle={{ color: '#3f8600' }}
            />
          </Card>
        </Col>
      </Row>

      <Row gutter={16}>
        <Col span={12}>
          <Card title="Recent Devices" extra={<Button type="link" onClick={fetchData}>Refresh</Button>}>
            <Table
              dataSource={devices.slice(0, 5)}
              columns={deviceColumns}
              pagination={false}
              size="small"
            />
          </Card>
        </Col>
        <Col span={12}>
          <Card title="System Status" extra={<Button type="link" onClick={fetchData}>Refresh</Button>}>
            <Space direction="vertical" style={{ width: '100%' }}>
              <div>
                <Text strong>Infrastructure:</Text> <Tag color="green">Running</Tag>
              </div>
              <div>
                <Text strong>API Server:</Text> <Tag color="green">Running</Tag>
              </div>
              <div>
                <Text strong>Dashboard:</Text> <Tag color="green">Running</Tag>
              </div>
              {gateways.map((gateway, index) => (
                <div key={index}>
                  <Text strong>{gateway.name}:</Text> <Tag color={gateway.status === 'Running' ? 'green' : gateway.status === 'Pending' ? 'orange' : 'red'}>{gateway.status}</Tag>
                </div>
              ))}
            </Space>
          </Card>
        </Col>
      </Row>
    </div>
  );


  const renderContent = () => {
    switch (selectedKey) {
      case 'devices':
        return <DeviceManagement />;
      case 'applications':
        return <ApplicationManagement />;
      case 'gateways':
        return <GatewayManagement />;
      case 'monitoring':
        return <Monitoring />;
      case 'terminal':
        return <Terminal />;
      case 'topology':
        return <NetworkTopology />;
      case 'configuration':
        return <InitialConfiguration />;
      case 'settings':
        return (
          <div>
            <Title level={2}>Settings</Title>
            <p>Settings page - Coming soon</p>
          </div>
        );
      default:
        return <Dashboard />;
    }
  };

  return (
    <Layout style={{ minHeight: '100vh' }}>
      <Sider width={200} style={{ background: '#fff' }}>
        <div style={{ padding: 16, textAlign: 'center', borderBottom: '1px solid #f0f0f0' }}>
          <Title level={4} style={{ margin: 0, color: '#1890ff' }}>
            Wasmbed
          </Title>
        </div>
        <Menu
          mode="inline"
          selectedKeys={[selectedKey]}
          style={{ height: '100%', borderRight: 0 }}
          onClick={({ key }) => setSelectedKey(key as string)}
        >
          <Menu.Item key="dashboard" icon={<DashboardOutlined />}>
            Dashboard
          </Menu.Item>
          <Menu.Item key="devices" icon={<MobileOutlined />}>
            Devices
          </Menu.Item>
          <Menu.Item key="applications" icon={<AppstoreOutlined />}>
            Applications
          </Menu.Item>
          <Menu.Item key="gateways" icon={<CloudServerOutlined />}>
            Gateways
          </Menu.Item>
          <Menu.Item key="monitoring" icon={<MonitorOutlined />}>
            Monitoring
          </Menu.Item>
          <Menu.Item key="terminal" icon={<ConsoleSqlOutlined />}>
            Terminal
          </Menu.Item>
          <Menu.Item key="topology" icon={<NodeIndexOutlined />}>
            Network Topology
          </Menu.Item>
          <Menu.Item key="configuration" icon={<RocketOutlined />}>
            Initial Configuration
          </Menu.Item>
          <Menu.Item key="settings" icon={<SettingOutlined />}>
            Settings
          </Menu.Item>
        </Menu>
      </Sider>
      <Layout>
        <Header style={{ background: '#fff', padding: '0 24px', borderBottom: '1px solid #f0f0f0' }}>
          <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
            <Title level={3} style={{ margin: 0 }}>
              {selectedKey === 'dashboard' && 'Dashboard'}
              {selectedKey === 'devices' && 'Device Management'}
              {selectedKey === 'applications' && 'Application Management'}
              {selectedKey === 'gateways' && 'Gateway Management'}
              {selectedKey === 'monitoring' && 'System Monitoring'}
              {selectedKey === 'terminal' && 'Terminal'}
              {selectedKey === 'topology' && 'Network Topology'}
              {selectedKey === 'configuration' && 'Initial Configuration'}
              {selectedKey === 'settings' && 'Settings'}
            </Title>
            <Space>
              <Button icon={<ReloadOutlined />} onClick={fetchData}>
                Refresh
              </Button>
            </Space>
          </div>
        </Header>
        <Content style={{ padding: 24, background: '#f5f5f5' }}>
          <Spin spinning={loading}>
            {renderContent()}
          </Spin>
        </Content>
      </Layout>
    </Layout>
  );
};

export default App;
