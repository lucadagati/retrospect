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
  message,
  Popconfirm,
  Typography,
  Row,
  Col,
  Statistic,
} from 'antd';
import {
  PlusOutlined,
  DeleteOutlined,
  ReloadOutlined,
  CheckCircleOutlined,
  ExclamationCircleOutlined,
  ClockCircleOutlined,
} from '@ant-design/icons';
import axios from 'axios';

const { Title } = Typography;
const { Option } = Select;

const DeviceManagement = () => {
  const [devices, setDevices] = useState([]);
  const [loading, setLoading] = useState(false);
  const [modalVisible, setModalVisible] = useState(false);
  const [form] = Form.useForm();

  useEffect(() => {
    fetchDevices();
  }, []);

  const fetchDevices = async () => {
    setLoading(true);
    try {
      const response = await axios.get('/api/devices');
      setDevices(response.data);
    } catch (error) {
      message.error('Failed to fetch devices');
      console.error('Error fetching devices:', error);
    } finally {
      setLoading(false);
    }
  };

  const handleCreateDevice = async (values) => {
    try {
      await axios.post('/api/devices', values);
      message.success('Device created successfully');
      setModalVisible(false);
      form.resetFields();
      fetchDevices();
    } catch (error) {
      message.error('Failed to create device');
      console.error('Error creating device:', error);
    }
  };

  const handleDeleteDevice = async (deviceId) => {
    try {
      await axios.delete(`/api/devices/${deviceId}`);
      message.success('Device deleted successfully');
      fetchDevices();
    } catch (error) {
      message.error('Failed to delete device');
      console.error('Error deleting device:', error);
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
      render: (_, record) => (
        <Space>
          <Popconfirm
            title="Are you sure you want to delete this device?"
            onConfirm={() => handleDeleteDevice(record.id)}
            okText="Yes"
            cancelText="No"
          >
            <Button type="link" danger icon={<DeleteOutlined />}>
              Delete
            </Button>
          </Popconfirm>
        </Space>
      ),
    },
  ];

  const deviceStats = devices.reduce(
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
            <Button
              type="primary"
              icon={<PlusOutlined />}
              onClick={() => setModalVisible(true)}
            >
              Add Device
            </Button>
            <Button
              icon={<ReloadOutlined />}
              onClick={fetchDevices}
              loading={loading}
            >
              Refresh
            </Button>
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
            label="Device Name"
            rules={[{ required: true, message: 'Please enter device name' }]}
          >
            <Input placeholder="Enter device name" />
          </Form.Item>

          <Form.Item
            name="architecture"
            label="Architecture"
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
            label="Device Type"
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
            label="Gateway Endpoint"
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
