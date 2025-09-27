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
  ReloadOutlined
} from '@ant-design/icons';
import './App.css';

const { Header, Content, Sider } = Layout;
const { Title, Text } = Typography;

interface Device {
  id: number;
  name: string;
  status: string;
  type: string;
  architecture: string;
  lastHeartbeat: string;
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

  // Mock data for development
  const mockDevices: Device[] = [
    { id: 1, name: 'mcu-board-1', status: 'Connected', type: 'MCU', architecture: 'riscv32', lastHeartbeat: '2025-09-27T17:30:00Z' },
    { id: 2, name: 'mcu-board-2', status: 'Connected', type: 'MCU', architecture: 'riscv32', lastHeartbeat: '2025-09-27T17:30:00Z' },
    { id: 3, name: 'mcu-board-3', status: 'Connected', type: 'MCU', architecture: 'riscv32', lastHeartbeat: '2025-09-27T17:30:00Z' },
    { id: 4, name: 'riscv-board-1', status: 'Connected', type: 'RISC-V', architecture: 'riscv64', lastHeartbeat: '2025-09-27T17:30:00Z' },
    { id: 5, name: 'riscv-board-2', status: 'Connected', type: 'RISC-V', architecture: 'riscv64', lastHeartbeat: '2025-09-27T17:30:00Z' },
    { id: 6, name: 'riscv-board-3', status: 'Connected', type: 'RISC-V', architecture: 'riscv64', lastHeartbeat: '2025-09-27T17:30:00Z' },
  ];

  const mockApplications: Application[] = [
    { id: 1, name: 'test-app-1', status: 'Running', targetDevices: ['mcu-board-1', 'mcu-board-2'], wasmBytes: '2.5KB' },
  ];

  const mockGateways: Gateway[] = [
    { id: 1, name: 'gateway-1', status: 'Active', endpoint: '127.0.0.1:30453', connectedDevices: 3, maxDevices: 10 },
    { id: 2, name: 'gateway-2', status: 'Active', endpoint: '127.0.0.1:30455', connectedDevices: 2, maxDevices: 10 },
    { id: 3, name: 'gateway-3', status: 'Active', endpoint: '127.0.0.1:30457', connectedDevices: 1, maxDevices: 10 },
  ];

  const fetchData = async () => {
    setLoading(true);
    try {
      // Use mock data for development
      setDevices(mockDevices);
      setApplications(mockApplications);
      setGateways(mockGateways);
    } catch (error) {
      console.error('Error fetching data:', error);
      // Use mock data on error
      setDevices(mockDevices);
      setApplications(mockApplications);
      setGateways(mockGateways);
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
      render: (status: string) => (
        <Tag color={status === 'Connected' ? 'green' : 'red'}>
          {status}
        </Tag>
      ),
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
    },
    {
      title: 'Last Heartbeat',
      dataIndex: 'lastHeartbeat',
      key: 'lastHeartbeat',
      render: (time: string) => new Date(time).toLocaleString(),
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
      render: (status: string) => (
        <Tag color={status === 'Running' ? 'green' : 'orange'}>
          {status}
        </Tag>
      ),
    },
    {
      title: 'Target Devices',
      dataIndex: 'targetDevices',
      key: 'targetDevices',
      render: (devices: string[]) => devices.join(', '),
    },
    {
      title: 'WASM Size',
      dataIndex: 'wasmBytes',
      key: 'wasmBytes',
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
      render: (status: string) => (
        <Tag color={status === 'Active' ? 'green' : 'red'}>
          {status}
        </Tag>
      ),
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
              value={applications.filter(app => app.status === 'Running').length}
              prefix={<AppstoreOutlined />}
              valueStyle={{ color: '#3f8600' }}
            />
          </Card>
        </Col>
        <Col span={6}>
          <Card>
            <Statistic
              title="Active Gateways"
              value={gateways.filter(gw => gw.status === 'Active').length}
              prefix={<CloudServerOutlined />}
              valueStyle={{ color: '#3f8600' }}
            />
          </Card>
        </Col>
        <Col span={6}>
          <Card>
            <Statistic
              title="Connected Devices"
              value={devices.filter(device => device.status === 'Connected').length}
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
                <Text strong>Gateway 1:</Text> <Tag color="green">Active</Tag>
              </div>
              <div>
                <Text strong>Gateway 2:</Text> <Tag color="green">Active</Tag>
              </div>
              <div>
                <Text strong>Gateway 3:</Text> <Tag color="green">Active</Tag>
              </div>
            </Space>
          </Card>
        </Col>
      </Row>
    </div>
  );

  const renderDevices = () => (
    <div>
      <div style={{ marginBottom: 16, display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
        <Title level={2}>Devices Management</Title>
        <Space>
          <Button type="primary" icon={<ReloadOutlined />} onClick={fetchData}>
            Refresh
          </Button>
        </Space>
      </div>
      <Table
        dataSource={devices}
        columns={deviceColumns}
        rowKey="name"
        loading={loading}
      />
    </div>
  );

  const renderApplications = () => (
    <div>
      <div style={{ marginBottom: 16, display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
        <Title level={2}>Applications Management</Title>
        <Space>
          <Button type="primary" icon={<PlayCircleOutlined />}>
            Deploy
          </Button>
          <Button icon={<ReloadOutlined />} onClick={fetchData}>
            Refresh
          </Button>
        </Space>
      </div>
      <Table
        dataSource={applications}
        columns={applicationColumns}
        rowKey="name"
        loading={loading}
      />
    </div>
  );

  const renderGateways = () => (
    <div>
      <div style={{ marginBottom: 16, display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
        <Title level={2}>Gateways Management</Title>
        <Space>
          <Button type="primary" icon={<ReloadOutlined />} onClick={fetchData}>
            Refresh
          </Button>
        </Space>
      </div>
      <Table
        dataSource={gateways}
        columns={gatewayColumns}
        rowKey="name"
        loading={loading}
      />
    </div>
  );

  const renderContent = () => {
    switch (selectedKey) {
      case 'devices':
        return renderDevices();
      case 'applications':
        return renderApplications();
      case 'gateways':
        return renderGateways();
      default:
        return renderDashboard();
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
              {selectedKey === 'devices' && 'Devices'}
              {selectedKey === 'applications' && 'Applications'}
              {selectedKey === 'gateways' && 'Gateways'}
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
