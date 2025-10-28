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
  DisconnectOutlined,
  PlayCircleOutlined,
  PauseCircleOutlined,
} from '@ant-design/icons';
import { apiGet, apiPost, apiDelete } from '../utils/api';

const { Title } = Typography;
const { Option } = Select;

const DeviceManagement = () => {
  const [devices, setDevices] = useState([]);
  const [renodeDevices, setRenodeDevices] = useState([]);
  const [gateways, setGateways] = useState([]);
  const [loading, setLoading] = useState(false);
  const [modalVisible, setModalVisible] = useState(false);
  const [form] = Form.useForm();

  // Initialize devices and gateways only once
  useEffect(() => {
    fetchDevices();
    fetchGateways();
    fetchRenodeDevices();
    
    // Set up auto-refresh for Renode devices every 5 seconds
    const interval = setInterval(() => {
      fetchRenodeDevices();
    }, 5000);
    
    return () => clearInterval(interval);
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

  const fetchRenodeDevices = async () => {
    try {
      const data = await apiGet('/api/v1/renode/devices', 10000);
      setRenodeDevices(data.devices || []);
    } catch (error) {
      console.error('Error fetching Renode devices:', error);
    }
  };

  const fetchGateways = async () => {
    try {
      const data = await apiGet('/api/v1/gateways', 10000);
      let gatewayList = data.gateways || [];
      setGateways(gatewayList);
    } catch (error) {
      console.error('Error fetching gateways:', error);
    }
  };

  const handleCreateDevice = async (values) => {
    try {
      // Auto-generate name if not provided
      const deviceCount = devices.length + 1;
      const deviceName = values.name || `device-${values.gateway || 'gateway-1'}-${deviceCount}`;
      
      const response = await apiPost('/api/v1/devices', {
        name: deviceName,
        type: 'MCU', // Fixed to MCU only
        architecture: 'ARM_CORTEX_M', // Fixed architecture for QEMU
        mcuType: values.mcuType || 'Mps2An385', // MCU type selection
        publicKey: 'auto-generated',
        gatewayId: values.gateway || 'gateway-1', // Use selected gateway
        qemuEnabled: true // Enable QEMU emulation
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
      const availableGateways = gateways.gateways?.filter(g => g.status === 'Running') || [];
      
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
            ? { ...device, status: data.status || 'Connected', connected: true, lastHeartbeat: data.lastHeartbeat }
            : device
        )
      );
      console.log(`Device ${deviceId} connected successfully`);
    } catch (error) {
      console.error('Error connecting device:', error);
    }
  };

  const handleDisconnectDevice = async (deviceId) => {
    try {
      const data = await apiPost(`/api/v1/devices/${deviceId}/disconnect`, {}, 10000);
      
      setDevices(prevDevices => 
        prevDevices.map(device => 
          device.id === deviceId 
            ? { ...device, status: data.status || 'Disconnected', connected: false }
            : device
        )
      );
      console.log(`Device ${deviceId} disconnected successfully`);
    } catch (error) {
      console.error('Error disconnecting device:', error);
    }
  };

  const handleStartRenode = async (deviceId) => {
    try {
      const data = await apiPost(`/api/v1/devices/${deviceId}/renode/start`, {}, 20000);
      
      setDevices(prevDevices => 
        prevDevices.map(device => 
          device.id === deviceId 
            ? { ...device, emulationStatus: 'Running', renodeInstance: data.renodeInstance }
            : device
        )
      );
      console.log(`Renode emulation started for device ${deviceId}`);
    } catch (error) {
      console.error('Error starting Renode emulation:', error);
    }
  };

  const handleStopRenode = async (deviceId) => {
    try {
      await apiPost(`/api/v1/devices/${deviceId}/renode/stop`, {}, 15000);
      
      setDevices(prevDevices => 
        prevDevices.map(device => 
          device.id === deviceId 
            ? { ...device, emulationStatus: 'Stopped', renodeInstance: null }
            : device
        )
      );
      console.log(`Renode emulation stopped for device ${deviceId}`);
    } catch (error) {
      console.error('Error stopping Renode emulation:', error);
    }
  };

  const getStatusTag = (status) => {
    const statusConfig = {
      Connected: { color: 'green', icon: <CheckCircleOutlined /> },
      Enrolled: { color: 'blue', icon: <CheckCircleOutlined /> },
      Enrolling: { color: 'cyan', icon: <ClockCircleOutlined /> },
      Pending: { color: 'orange', icon: <ClockCircleOutlined /> },
      Disconnected: { color: 'gray', icon: <ExclamationCircleOutlined /> },
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
      sorter: (a, b) => (a.name || '').localeCompare(b.name || ''),
    },
    {
      title: 'Status',
      dataIndex: 'status',
      key: 'status',
      render: (status) => getStatusTag(status),
      filters: [
        { text: 'Connected', value: 'Connected', key: 'connected' },
        { text: 'Enrolled', value: 'Enrolled', key: 'enrolled' },
        { text: 'Enrolling', value: 'Enrolling', key: 'enrolling' },
        { text: 'Pending', value: 'Pending', key: 'pending' },
        { text: 'Disconnected', value: 'Disconnected', key: 'disconnected' },
        { text: 'Unreachable', value: 'Unreachable', key: 'unreachable' },
      ],
      onFilter: (value, record) => record.status === value,
    },
    {
      title: 'Architecture',
      dataIndex: 'architecture',
      key: 'architecture',
      render: (arch) => (
        <Tag color="blue">ARM Cortex-M (Renode)</Tag>
      ),
    },
    {
      title: 'Device Type',
      dataIndex: 'type',
      key: 'type',
      render: (type) => (
        <Tag color="green">{type}</Tag>
      ),
      filters: [
        { text: 'MCU', value: 'MCU', key: 'mcu' },
        { text: 'MPU', value: 'MPU', key: 'mpu' },
        { text: 'RISC-V', value: 'RISC-V', key: 'riscv' },
      ],
      onFilter: (value, record) => record.type === value,
    },
    {
      title: 'MCU Type',
      dataIndex: 'mcuType',
      key: 'mcuType',
      render: (mcuType) => {
        const mcuNames = {
          'Mps2An385': 'MPS2-AN385',
          'Mps2An386': 'MPS2-AN386', 
          'Mps2An500': 'MPS2-AN500',
          'Mps2An505': 'MPS2-AN505',
          'Stm32Vldiscovery': 'STM32VL-Discovery',
          'OlimexStm32H405': 'Olimex STM32-H405'
        };
        return <Tag color="purple">{mcuNames[mcuType] || mcuType}</Tag>;
      },
    },
    {
      title: 'Gateway',
      dataIndex: 'gateway',
      key: 'gateway',
      render: (gateway) => (
        <Tag color="orange">{gateway}</Tag>
      ),
    },
    {
      title: 'Last Heartbeat',
      dataIndex: 'lastHeartbeat',
      key: 'lastHeartbeat',
      render: (timestamp) => timestamp ? new Date(timestamp * 1000).toLocaleString() : 'Never',
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
          {record.status === 'Connected' && (
            <Tooltip key="disconnect" title="Disconnect device from gateway">
              <Button 
                type="link" 
                icon={<DisconnectOutlined />}
                size="small"
                onClick={() => handleDisconnectDevice(record.id)}
              >
                Disconnect
              </Button>
            </Tooltip>
          )}
          {record.emulationStatus === 'Not Started' || record.emulationStatus === 'Stopped' ? (
            <Tooltip key="start-renode" title="Start Renode emulation">
              <Button 
                type="link" 
                icon={<PlayCircleOutlined />}
                size="small"
                onClick={() => handleStartRenode(record.id)}
              >
                Start Renode
              </Button>
            </Tooltip>
          ) : (
            <Tooltip key="stop-renode" title="Stop Renode emulation">
              <Button 
                type="link" 
                icon={<PauseCircleOutlined />}
                size="small"
                onClick={() => handleStopRenode(record.id)}
              >
                Stop Renode
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
      else if (device.status === 'Enrolling') acc.enrolling++;
      else if (device.status === 'Pending') acc.pending++;
      else if (device.status === 'Disconnected') acc.disconnected++;
      else if (device.status === 'Unreachable') acc.unreachable++;
      return acc;
    },
    { total: 0, connected: 0, enrolled: 0, enrolling: 0, pending: 0, disconnected: 0, unreachable: 0 }
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
              title="Enrolling"
              value={deviceStats.enrolling}
              valueStyle={{ color: '#13c2c2' }}
            />
          </Card>
        </Col>
        <Col xs={12} sm={6}>
          <Card size="small">
            <Statistic
              title="Disconnected"
              value={deviceStats.disconnected}
              valueStyle={{ color: '#8c8c8c' }}
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

        {/* Renode Devices Section */}
        <Card 
          title="Renode Devices" 
          size="small" 
          style={{ marginBottom: 16 }}
          extra={
            <Button 
              icon={<ReloadOutlined />} 
              size="small"
              onClick={fetchRenodeDevices}
            >
              Refresh Renode
            </Button>
          }
        >
          <Row gutter={[16, 16]}>
            {renodeDevices.map((device, index) => (
              <Col xs={24} sm={12} md={8} lg={6} key={device.id || index}>
                <Card size="small" hoverable>
                  <div style={{ textAlign: 'center' }}>
                    <div style={{ fontSize: '16px', fontWeight: 'bold', marginBottom: 8 }}>
                      {device.name}
                    </div>
                    <Tag color={device.status === 'Running' ? 'green' : 'default'}>
                      {device.status}
                    </Tag>
                    <div style={{ fontSize: '12px', color: '#666', marginTop: 4 }}>
                      {device.mcu_type} • {device.architecture}
                    </div>
                    <div style={{ fontSize: '11px', color: '#999', marginTop: 2 }}>
                      {device.endpoint}
                    </div>
                  </div>
                </Card>
              </Col>
            ))}
            {renodeDevices.length === 0 && (
              <Col span={24}>
                <div style={{ textAlign: 'center', color: '#999', padding: '20px 0' }}>
                  No Renode devices available
                </div>
              </Col>
            )}
          </Row>
        </Card>

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
            label="Architecture"
            help="Fixed to ARM Cortex-M for Renode emulation"
          >
            <Select placeholder="ARM Cortex-M (Renode)" disabled>
              <Option value="ARM_CORTEX_M">ARM Cortex-M (Renode)</Option>
            </Select>
          </Form.Item>

          <Form.Item
            name="deviceType"
            label="Device Type"
            help="Fixed to MCU for edge computing"
          >
            <Select placeholder="MCU" disabled>
              <Option value="MCU">MCU</Option>
            </Select>
          </Form.Item>

          <Form.Item
            name="mcuType"
            label="MCU Type"
            help="Select the specific MCU for Renode emulation"
            rules={[{ required: true, message: 'Please select an MCU type!' }]}
          >
            <Select placeholder="Select MCU Type">
              <Option value="Mps2An385">ARM MPS2-AN385 (Cortex-M3) - Default, most compatible</Option>
              <Option value="Mps2An386">ARM MPS2-AN386 (Cortex-M4) - Enhanced with FPU</Option>
              <Option value="Mps2An500">ARM MPS2-AN500 (Cortex-M7) - High performance</Option>
              <Option value="Mps2An505">ARM MPS2-AN505 (Cortex-M33) - TrustZone support</Option>
              <Option value="Stm32Vldiscovery">STM32VLDISCOVERY (Cortex-M3) - STMicroelectronics</Option>
              <Option value="OlimexStm32H405">Olimex STM32-H405 (Cortex-M4) - Olimex board</Option>
            </Select>
          </Form.Item>

          <Form.Item
            name="gateway"
            label="Target Gateway"
            help="Select which gateway this device should connect to"
            rules={[{ required: true, message: 'Please select a gateway!' }]}
          >
            <Select placeholder="Select Gateway">
              {gateways.map((gateway) => (
                <Option key={gateway.id} value={gateway.id}>
                  {gateway.name} ({gateway.status})
                </Option>
              ))}
            </Select>
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
