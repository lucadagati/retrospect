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
      const response = await fetch('/api/v1/devices');
      if (response.ok) {
        const data = await response.json();
        let deviceList = data.devices || [];
        
        // Use real data from backend - no mock data
        
        setDevices(deviceList);
      } else {
        console.error('Failed to fetch devices:', response.status);
      }
    } catch (error) {
      console.error('Error fetching devices:', error);
    } finally {
      setLoading(false);
    }
  };

  const handleCreateDevice = async (values) => {
    try {
      const response = await fetch('/api/v1/devices', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          name: values.name,
          type: values.type,
          architecture: values.architecture,
          publicKey: values.publicKey || 'auto-generated'
        })
      });
      
      if (response.ok) {
        const newDevice = await response.json();
        setDevices(prevDevices => [...prevDevices, newDevice]);
        console.log('Device created successfully:', newDevice.name);
        setModalVisible(false);
        form.resetFields();
      } else {
        console.error('Failed to create device:', response.status);
      }
    } catch (error) {
      console.error('Error creating device:', error);
    }
  };

  const handleDeleteDevice = async (deviceId) => {
    try {
      const response = await fetch(`/api/v1/devices/${deviceId}`, {
        method: 'DELETE'
      });
      
      if (response.ok) {
        setDevices(prevDevices => prevDevices.filter(device => device.id !== deviceId));
        console.log('Device deleted successfully:', deviceId);
      } else {
        console.error('Failed to delete device:', response.status);
      }
    } catch (error) {
      console.error('Error deleting device:', error);
    }
  };

  const handleEnrollDevice = async (deviceId) => {
    try {
      // First, get available gateways
      const gatewaysResponse = await fetch('/api/v1/gateways');
      if (!gatewaysResponse.ok) {
        console.error('Failed to fetch gateways for enrollment');
        return;
      }
      
      const gateways = await gatewaysResponse.json();
      const availableGateways = gateways.gateways?.filter(g => g.status === 'Active') || [];
      
      if (availableGateways.length === 0) {
        console.error('No active gateways available for enrollment');
        return;
      }
      
      // For now, select the first available gateway
      // In a real implementation, this could be a user selection
      const selectedGateway = availableGateways[0];
      
      const response = await fetch(`/api/v1/devices/${deviceId}/enroll`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          gatewayId: selectedGateway.id,
          gatewayName: selectedGateway.name
        })
      });
      
      if (response.ok) {
        const data = await response.json();
        setDevices(prevDevices => 
          prevDevices.map(device => 
            device.id === deviceId 
              ? { ...device, enrolled: true, status: 'Enrolled', gatewayId: data.gatewayId, gatewayName: data.gatewayName }
              : device
          )
        );
        console.log(`Device ${deviceId} enrolled successfully to gateway ${selectedGateway.name}`);
      } else {
        console.error('Failed to enroll device:', response.status);
      }
    } catch (error) {
      console.error('Error enrolling device:', error);
    }
  };

  const handleConnectDevice = async (deviceId) => {
    try {
      const response = await fetch(`/api/v1/devices/${deviceId}/connect`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        }
      });
      
      if (response.ok) {
        const data = await response.json();
        setDevices(prevDevices => 
          prevDevices.map(device => 
            device.id === deviceId 
              ? { ...device, status: 'Connected', connected: true, lastHeartbeat: data.lastHeartbeat }
              : device
          )
        );
        console.log(`Device ${deviceId} connected successfully`);
      } else {
        console.error('Failed to connect device:', response.status);
      }
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
      dataIndex: 'name',
      key: 'name',
      sorter: (a, b) => a.name.localeCompare(b.name),
    },
    {
      title: 'Status',
      dataIndex: 'status',
      key: 'status',
      render: (status) => getStatusTag(status),
      filters: [
        { text: 'Connected', value: 'Connected' },
        { text: 'Enrolled', value: 'Enrolled' },
        { text: 'Pending', value: 'Pending' },
        { text: 'Failed', value: 'Failed' },
        { text: 'Unreachable', value: 'Unreachable' },
      ],
      onFilter: (value, record) => record.status === value,
    },
    {
      title: 'Architecture',
      dataIndex: 'architecture',
      key: 'architecture',
      filters: [
        { text: 'arm64', value: 'arm64' },
        { text: 'x86_64', value: 'x86_64' },
        { text: 'riscv64', value: 'riscv64' },
      ],
      onFilter: (value, record) => record.architecture === value,
    },
    {
      title: 'Device Type',
      dataIndex: 'deviceType',
      key: 'deviceType',
      filters: [
        { text: 'MCU', value: 'MCU' },
        { text: 'MPU', value: 'MPU' },
        { text: 'RISC-V', value: 'RISC-V' },
      ],
      onFilter: (value, record) => record.deviceType === value,
    },
    {
      title: 'Gateway',
      dataIndex: 'gateway',
      key: 'gateway',
    },
    {
      title: 'Last Heartbeat',
      dataIndex: 'lastHeartbeat',
      key: 'lastHeartbeat',
      render: (timestamp) => timestamp ? new Date(timestamp).toLocaleString() : 'Never',
    },
    {
      title: 'Actions',
      key: 'actions',
      width: 200,
      fixed: 'right',
      render: (_, record) => (
        <Space size="small">
          {record.status === 'Disconnected' && (
            <Tooltip title="Enroll device in the system">
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
            <Tooltip title="Connect device to gateway">
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
          rowKey="id"
          loading={loading}
          pagination={{
            pageSize: 10,
            showSizeChanger: true,
            showQuickJumper: true,
            showTotal: (total, range) =>
              `${range[0]}-${range[1]} of ${total} devices`,
          }}
          scroll={{ x: 1000 }}
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
            label={
              <Space>
                <span>Device Name</span>
                <Tooltip title="Unique identifier for the device (e.g., mcu-board-1, riscv-board-2)">
                  <QuestionCircleOutlined style={{ color: '#1890ff' }} />
                </Tooltip>
              </Space>
            }
            rules={[{ required: true, message: 'Please enter device name' }]}
          >
            <Input placeholder="Enter device name" />
          </Form.Item>

          <Form.Item
            name="architecture"
            label={
              <Space>
                <span>Architecture</span>
                <Tooltip title="CPU architecture of the device (ARM64, x86_64, RISC-V 64)">
                  <QuestionCircleOutlined style={{ color: '#1890ff' }} />
                </Tooltip>
              </Space>
            }
            rules={[{ required: true, message: 'Please select architecture' }]}
          >
            <Select placeholder="Select architecture">
              <Option value="arm64">ARM64</Option>
              <Option value="x86_64">x86_64</Option>
              <Option value="riscv64">RISC-V 64</Option>
            </Select>
          </Form.Item>

          <Form.Item
            name="deviceType"
            label={
              <Space>
                <span>Device Type</span>
                <Tooltip title="Type of device (MCU: Microcontroller, MPU: Microprocessor, RISC-V: RISC-V processor)">
                  <QuestionCircleOutlined style={{ color: '#1890ff' }} />
                </Tooltip>
              </Space>
            }
            rules={[{ required: true, message: 'Please select device type' }]}
          >
            <Select placeholder="Select device type">
              <Option value="MCU">MCU</Option>
              <Option value="MPU">MPU</Option>
              <Option value="RISC-V">RISC-V</Option>
            </Select>
          </Form.Item>

          <Form.Item
            name="gatewayEndpoint"
            label={
              <Space>
                <span>Gateway Endpoint</span>
                <Tooltip title="Network endpoint where the device will connect to the gateway">
                  <QuestionCircleOutlined style={{ color: '#1890ff' }} />
                </Tooltip>
              </Space>
            }
            rules={[{ required: true, message: 'Please enter gateway endpoint' }]}
          >
            <Input placeholder="gateway-1.wasmbed.svc.cluster.local:30430" />
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
