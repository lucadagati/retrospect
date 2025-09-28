import React, { useState, useEffect } from 'react';
import {
  Table,
  Card,
  Button,
  Space,
  Tag,
  Modal,
  Form,
  Input,
  Select,
  Popconfirm,
  Typography,
  Row,
  Col,
  Statistic,
  Tooltip,
} from 'antd';
import {
  PlusOutlined,
  DeleteOutlined,
  ReloadOutlined,
  CheckCircleOutlined,
  ExclamationCircleOutlined,
  ClockCircleOutlined,
  QuestionCircleOutlined,
  UserAddOutlined,
  LinkOutlined,
} from '@ant-design/icons';
import { apiGet, apiPost, apiDelete } from '../utils/api';

const { Title } = Typography;
const { Option } = Select;

const DeviceManagement = () => {
  const [devices, setDevices] = useState([]);
  const [loading, setLoading] = useState(false);
  const [modalVisible, setModalVisible] = useState(false);
  const [form] = Form.useForm();

  // Initialize devices only once
  useEffect(() => {
    fetchDevices();
  }, []);

  const fetchDevices = async () => {
    setLoading(true);
    try {
      const data = await apiGet('/api/v1/devices', 10000);
      let deviceList = data.devices || [];
      
      // Use real data from backend - no mock data
      setDevices(deviceList);
    } catch (error) {
      console.error('Error fetching devices:', error);
    } finally {
      setLoading(false);
    }
  };

  const handleCreateDevice = async (values) => {
    try {
      // Auto-generate name if not provided
      const deviceCount = devices.length + 1;
      const deviceName = values.name || `device-gateway-1-${deviceCount}`;
      
      // Auto-generate gateway endpoint if not provided
      const gatewayEndpoint = values.gatewayEndpoint || 'gateway-1.wasmbed.svc.cluster.local:30430';
      
      const response = await apiPost('/api/v1/devices', {
        name: deviceName,
        type: values.deviceType || 'MCU',
        architecture: values.architecture || 'riscv32',
        publicKey: 'auto-generated',
        gatewayEndpoint: gatewayEndpoint
      }, 15000);
      
      // Refresh the devices list to get the updated data
      const updatedDevices = await apiGet('/api/v1/devices');
      setDevices(updatedDevices.devices || []);
      console.log('Device created successfully:', response.message);
      setModalVisible(false);
      form.resetFields();
    } catch (error) {
      console.error('Error creating device:', error);
    }
  };

  const handleDeleteDevice = async (deviceId) => {
    try {
      await apiDelete(`/api/v1/devices/${deviceId}`, 10000);
      setDevices(prevDevices => prevDevices.filter(device => device.id !== deviceId));
      console.log('Device deleted successfully:', deviceId);
    } catch (error) {
      console.error('Error deleting device:', error);
    }
  };

  const handleEnrollDevice = async (deviceId) => {
    try {
      // First, get available gateways
      const gateways = await apiGet('/api/v1/gateways', 10000);
      const availableGateways = gateways.gateways?.filter(g => g.status === 'Active') || [];
      
      if (availableGateways.length === 0) {
        console.error('No active gateways available for enrollment');
        return;
      }
      
      // For now, select the first available gateway
      // In a real implementation, this could be a user selection
      const selectedGateway = availableGateways[0];
      
      const data = await apiPost(`/api/v1/devices/${deviceId}/enroll`, {
        gatewayId: selectedGateway.id,
        gatewayName: selectedGateway.name
      }, 15000);
      
      setDevices(prevDevices => 
        prevDevices.map(device => 
          device.id === deviceId 
            ? { ...device, enrolled: true, status: 'Enrolled', gatewayId: data.gatewayId, gatewayName: data.gatewayName }
            : device
        )
      );
      console.log(`Device ${deviceId} enrolled successfully to gateway ${selectedGateway.name}`);
    } catch (error) {
      console.error('Error enrolling device:', error);
    }
  };

  const handleConnectDevice = async (deviceId) => {
    try {
      const data = await apiPost(`/api/v1/devices/${deviceId}/connect`, {}, 15000);
      
      setDevices(prevDevices => 
        prevDevices.map(device => 
          device.id === deviceId 
            ? { ...device, status: 'Connected', connected: true, lastHeartbeat: data.lastHeartbeat }
            : device
        )
      );
      console.log(`Device ${deviceId} connected successfully`);
    } catch (error) {
      console.error('Error connecting device:', error);
    }
  };

  const getStatusTag = (status) => {
    const statusConfig = {
      Connected: { color: 'green', icon: <CheckCircleOutlined /> },
      Enrolled: { color: 'blue', icon: <CheckCircleOutlined /> },
      Pending: { color: 'orange', icon: <ClockCircleOutlined /> },
      Failed: { color: 'red', icon: <ExclamationCircleOutlined /> },
      Unreachable: { color: 'red', icon: <ExclamationCircleOutlined /> },
    };

    const config = statusConfig[status] || { color: 'default', icon: null };
    return (
      <Tag color={config.color} icon={config.icon}>
        {status}
      </Tag>
    );
  };

  const columns = [
    {
      title: 'Name',
      dataIndex: 'device_id',
      key: 'device_id',
      sorter: (a, b) => (a.device_id || '').localeCompare(b.device_id || ''),
    },
    {
      title: 'Status',
      dataIndex: 'status',
      key: 'status',
      render: (status) => getStatusTag(status),
      filters: [
        { text: 'Connected', value: 'Connected', key: 'connected' },
        { text: 'Enrolled', value: 'Enrolled', key: 'enrolled' },
        { text: 'Pending', value: 'Pending', key: 'pending' },
        { text: 'Failed', value: 'Failed', key: 'failed' },
        { text: 'Unreachable', value: 'Unreachable', key: 'unreachable' },
      ],
      onFilter: (value, record) => record.status === value,
    },
    {
      title: 'Architecture',
      dataIndex: 'architecture',
      key: 'architecture',
      filters: [
        { text: 'arm64', value: 'arm64', key: 'arm64' },
        { text: 'x86_64', value: 'x86_64', key: 'x86_64' },
        { text: 'riscv64', value: 'riscv64', key: 'riscv64' },
      ],
      onFilter: (value, record) => record.architecture === value,
    },
    {
      title: 'Device Type',
      dataIndex: 'device_type',
      key: 'device_type',
      filters: [
        { text: 'MCU', value: 'MCU', key: 'mcu' },
        { text: 'MPU', value: 'MPU', key: 'mpu' },
        { text: 'RISC-V', value: 'RISC-V', key: 'riscv' },
      ],
      onFilter: (value, record) => record.device_type === value,
    },
    {
      title: 'Gateway',
      dataIndex: 'gateway_id',
      key: 'gateway_id',
    },
    {
      title: 'Last Heartbeat',
      dataIndex: 'last_heartbeat',
      key: 'last_heartbeat',
      render: (timestamp) => timestamp ? new Date(timestamp.secs_since_epoch * 1000).toLocaleString() : 'Never',
    },
    {
      title: 'Actions',
      key: 'actions',
      width: 200,
      fixed: 'right',
      render: (_, record) => (
        <Space size="small">
          {record.status === 'Disconnected' && (
            <Tooltip key="enroll" title="Enroll device in the system">
              <Button 
                type="link" 
                icon={<UserAddOutlined />}
                size="small"
                onClick={() => handleEnrollDevice(record.id)}
              >
                Enroll
              </Button>
            </Tooltip>
          )}
          {record.status === 'Enrolled' && (
            <Tooltip key="connect" title="Connect device to gateway">
              <Button 
                type="link" 
                icon={<LinkOutlined />}
                size="small"
                onClick={() => handleConnectDevice(record.id)}
              >
                Connect
              </Button>
            </Tooltip>
          )}
          <Popconfirm
            key="delete"
            title="Are you sure you want to delete this device?"
            onConfirm={() => handleDeleteDevice(record.id)}
            okText="Yes"
            cancelText="No"
          >
            <Button 
              type="link" 
              danger 
              icon={<DeleteOutlined />}
              size="small"
            >
              Delete
            </Button>
          </Popconfirm>
        </Space>
      ),
    },
  ];

  const deviceStats = (Array.isArray(devices) ? devices : []).reduce(
    (acc, device) => {
      acc.total++;
      if (device.status === 'Connected') acc.connected++;
      else if (device.status === 'Enrolled') acc.enrolled++;
      else if (device.status === 'Pending') acc.pending++;
      else if (device.status === 'Failed') acc.failed++;
      else if (device.status === 'Unreachable') acc.unreachable++;
      return acc;
    },
    { total: 0, connected: 0, enrolled: 0, pending: 0, failed: 0, unreachable: 0 }
  );

  return (
    <div>
      <Title level={2}>Device Management</Title>
      
      <Row gutter={[16, 16]} style={{ marginBottom: 24 }}>
        <Col xs={12} sm={6}>
          <Card size="small">
            <Statistic
              title="Total Devices"
              value={deviceStats.total}
              valueStyle={{ color: '#1890ff' }}
            />
          </Card>
        </Col>
        <Col xs={12} sm={6}>
          <Card size="small">
            <Statistic
              title="Connected"
              value={deviceStats.connected}
              valueStyle={{ color: '#3f8600' }}
            />
          </Card>
        </Col>
        <Col xs={12} sm={6}>
          <Card size="small">
            <Statistic
              title="Enrolled"
              value={deviceStats.enrolled}
              valueStyle={{ color: '#722ed1' }}
            />
          </Card>
        </Col>
        <Col xs={12} sm={6}>
          <Card size="small">
            <Statistic
              title="Unreachable"
              value={deviceStats.unreachable}
              valueStyle={{ color: '#cf1322' }}
            />
          </Card>
        </Col>
      </Row>

      <Card>
        <div style={{ marginBottom: 16 }}>
          <Space>
            <Tooltip title="Create a new device with custom configuration">
              <Button
                type="primary"
                icon={<PlusOutlined />}
                onClick={() => setModalVisible(true)}
              >
                Add Device
              </Button>
            </Tooltip>
            <Tooltip title="Refresh the device list to get the latest status">
              <Button
                icon={<ReloadOutlined />}
                onClick={fetchDevices}
                loading={loading}
              >
                Refresh
              </Button>
            </Tooltip>
          </Space>
        </div>

        <Table
          columns={columns}
          dataSource={devices}
          rowKey={(record) => record.device_id || record.id || Math.random()}
          loading={loading}
          pagination={{
            pageSize: 10,
            showSizeChanger: true,
            showQuickJumper: true,
            showTotal: (total, range) =>
              `${range[0]}-${range[1]} of ${total} devices`,
          }}
          scroll={{ x: 'max-content' }}
          size="small"
        />
      </Card>

      <Modal
        title="Create New Device"
        open={modalVisible}
        onCancel={() => {
          setModalVisible(false);
          form.resetFields();
        }}
        footer={null}
      >
        <Form
          form={form}
          layout="vertical"
          onFinish={handleCreateDevice}
        >
          <Form.Item
            name="name"
            label="Device Name (Optional)"
            help="Leave empty for auto-generated name"
          >
            <Input placeholder="Auto-generated if empty" />
          </Form.Item>

          <Form.Item
            name="architecture"
            label="Architecture (Optional)"
            help="Leave empty for default RISC-V 32"
          >
            <Select placeholder="Default: RISC-V 32" allowClear>
              <Option value="riscv32">RISC-V 32</Option>
              <Option value="arm64">ARM64</Option>
              <Option value="x86_64">x86_64</Option>
              <Option value="riscv64">RISC-V 64</Option>
            </Select>
          </Form.Item>

          <Form.Item
            name="deviceType"
            label="Device Type (Optional)"
            help="Leave empty for default MCU"
          >
            <Select placeholder="Default: MCU" allowClear>
              <Option value="MCU">MCU</Option>
              <Option value="MPU">MPU</Option>
              <Option value="RISC-V">RISC-V</Option>
            </Select>
          </Form.Item>

          <Form.Item
            name="gatewayEndpoint"
            label="Gateway Endpoint (Optional)"
            help="Leave empty for auto-generated endpoint"
          >
            <Input placeholder="Auto-generated if empty" />
          </Form.Item>

          <Form.Item>
            <Space>
              <Button type="primary" htmlType="submit">
                Create Device
              </Button>
              <Button onClick={() => {
                setModalVisible(false);
                form.resetFields();
              }}>
                Cancel
              </Button>
            </Space>
          </Form.Item>
        </Form>
      </Modal>
    </div>
  );
};

export default DeviceManagement;
