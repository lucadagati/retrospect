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
  Descriptions,
  Switch,
  Tooltip,
} from 'antd';
import {
  PlusOutlined,
  DeleteOutlined,
  ReloadOutlined,
  CheckCircleOutlined,
  ExclamationCircleOutlined,
  ClockCircleOutlined,
  SettingOutlined,
  QuestionCircleOutlined,
} from '@ant-design/icons';

const { Title } = Typography;
const { Option } = Select;

const GatewayManagement = () => {
  const [gateways, setGateways] = useState([]);
  const [loading, setLoading] = useState(false);
  const [modalVisible, setModalVisible] = useState(false);
  const [configModalVisible, setConfigModalVisible] = useState(false);
  const [selectedGateway, setSelectedGateway] = useState(null);
  const [form] = Form.useForm();
  const [configForm] = Form.useForm();

  // Initialize data only once
  useEffect(() => {
    fetchGateways();
  }, []);

  const fetchGateways = async () => {
    setLoading(true);
    try {
      const response = await fetch('/api/v1/gateways');
      if (response.ok) {
        const data = await response.json();
        let gatewayList = data.gateways || [];
        
        // Use real data from backend - no mock data
        
        setGateways(gatewayList);
      } else {
        console.error('Failed to fetch gateways:', response.status);
      }
    } catch (error) {
      console.error('Error fetching gateways:', error);
    } finally {
      setLoading(false);
    }
  };

  const handleCreateGateway = async (values) => {
    try {
      const response = await fetch('/api/v1/gateways', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          name: values.name,
          endpoint: values.endpoint,
          maxDevices: values.maxDevices || 10,
          region: values.region || 'us-west-1'
        })
      });
      
      if (response.ok) {
        const newGateway = await response.json();
        setGateways(prevGateways => [...prevGateways, newGateway]);
        console.log('Gateway created successfully:', newGateway.name);
        setModalVisible(false);
        form.resetFields();
      } else {
        console.error('Failed to create gateway:', response.status);
      }
    } catch (error) {
      console.error('Error creating gateway:', error);
    }
  };

  const handleDeleteGateway = async (gatewayId) => {
    try {
      const response = await fetch(`/api/v1/gateways/${gatewayId}`, {
        method: 'DELETE'
      });
      
      if (response.ok) {
        setGateways(prevGateways => prevGateways.filter(gateway => gateway.id !== gatewayId));
        console.log('Gateway deleted successfully:', gatewayId);
      } else {
        console.error('Failed to delete gateway:', response.status);
      }
    } catch (error) {
      console.error('Error deleting gateway:', error);
    }
  };

  const handleUpdateGatewayConfig = async (values) => {
    try {
      const response = await fetch(`/api/v1/gateways/${selectedGateway.id}`, {
        method: 'PUT',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify(values)
      });
      
      if (response.ok) {
        const updatedGateway = await response.json();
        setGateways(prevGateways => 
          prevGateways.map(gateway => 
            gateway.id === selectedGateway.id 
              ? updatedGateway
              : gateway
          )
        );
        console.log('Gateway configuration updated successfully');
        setConfigModalVisible(false);
        configForm.resetFields();
        setSelectedGateway(null);
      } else {
        console.error('Failed to update gateway configuration:', response.status);
      }
    } catch (error) {
      console.error('Error updating gateway configuration:', error);
    }
  };

  const handleToggleGateway = async (gatewayId, enabled) => {
    try {
      const response = await fetch(`/api/v1/gateways/${gatewayId}/toggle`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({ enabled })
      });
      
      if (response.ok) {
        const updatedGateway = await response.json();
        setGateways(prevGateways => 
          prevGateways.map(gateway => 
            gateway.id === gatewayId 
              ? updatedGateway
              : gateway
          )
        );
        console.log(`Gateway ${enabled ? 'enabled' : 'disabled'} successfully:`, gatewayId);
      } else {
        console.error('Failed to update gateway status:', response.status);
      }
    } catch (error) {
      console.error('Error toggling gateway:', error);
    }
  };

  const getStatusTag = (status) => {
    const statusConfig = {
      Active: { color: 'green', icon: <CheckCircleOutlined /> },
      Inactive: { color: 'red', icon: <ExclamationCircleOutlined /> },
      Pending: { color: 'orange', icon: <ClockCircleOutlined /> },
      Degraded: { color: 'purple', icon: <ExclamationCircleOutlined /> },
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
        { text: 'Active', value: 'Active' },
        { text: 'Inactive', value: 'Inactive' },
        { text: 'Pending', value: 'Pending' },
        { text: 'Degraded', value: 'Degraded' },
      ],
      onFilter: (value, record) => record.status === value,
    },
    {
      title: 'Endpoint',
      dataIndex: 'endpoint',
      key: 'endpoint',
      ellipsis: true,
    },
    {
      title: 'Connected Devices',
      dataIndex: 'connectedDevices',
      key: 'connectedDevices',
      render: (count) => (
        <Statistic
          value={count || 0}
          valueStyle={{ fontSize: '14px', color: count > 0 ? '#3f8600' : '#999' }}
        />
      ),
    },
    {
      title: 'Last Heartbeat',
      dataIndex: 'lastHeartbeat',
      key: 'lastHeartbeat',
      render: (timestamp) => timestamp ? new Date(timestamp).toLocaleString() : 'Never',
    },
    {
      title: 'Enabled',
      dataIndex: 'enabled',
      key: 'enabled',
      render: (enabled, record) => (
        <Switch
          checked={enabled}
          onChange={(checked) => handleToggleGateway(record.id, checked)}
          checkedChildren="Active"
          unCheckedChildren="Inactive"
        />
      ),
    },
    {
      title: 'Actions',
      key: 'actions',
      width: 150,
      fixed: 'right',
      render: (_, record) => (
        <Space size="small">
          <Button
            type="link"
            icon={<SettingOutlined />}
            onClick={() => {
              setSelectedGateway(record);
              configForm.setFieldsValue({
                heartbeatInterval: record.config?.heartbeatInterval || '30s',
                connectionTimeout: record.config?.connectionTimeout || '10m',
                enrollmentTimeout: record.config?.enrollmentTimeout || '5m',
              });
              setConfigModalVisible(true);
            }}
            size="small"
          >
            Configure
          </Button>
          <Popconfirm
            title="Are you sure you want to delete this gateway?"
            onConfirm={() => handleDeleteGateway(record.id)}
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

  const gatewayStats = gateways.reduce(
    (acc, gateway) => {
      acc.total++;
      if (gateway.status === 'Active') acc.active++;
      else if (gateway.status === 'Inactive') acc.inactive++;
      else if (gateway.status === 'Pending') acc.pending++;
      else if (gateway.status === 'Degraded') acc.degraded++;
      acc.totalDevices += gateway.connectedDevices || 0;
      return acc;
    },
    { total: 0, active: 0, inactive: 0, pending: 0, degraded: 0, totalDevices: 0 }
  );

  return (
    <div>
      <Title level={2}>Gateway Management</Title>
      
      <Row gutter={[16, 16]} style={{ marginBottom: 24 }}>
        <Col xs={12} sm={6}>
          <Card size="small">
            <Statistic
              title="Total Gateways"
              value={gatewayStats.total}
              valueStyle={{ color: '#1890ff' }}
            />
          </Card>
        </Col>
        <Col xs={12} sm={6}>
          <Card size="small">
            <Statistic
              title="Active"
              value={gatewayStats.active}
              valueStyle={{ color: '#3f8600' }}
            />
          </Card>
        </Col>
        <Col xs={12} sm={6}>
          <Card size="small">
            <Statistic
              title="Inactive"
              value={gatewayStats.inactive}
              valueStyle={{ color: '#cf1322' }}
            />
          </Card>
        </Col>
        <Col xs={12} sm={6}>
          <Card size="small">
            <Statistic
              title="Total Devices"
              value={gatewayStats.totalDevices}
              valueStyle={{ color: '#722ed1' }}
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
              Add Gateway
            </Button>
            <Button
              icon={<ReloadOutlined />}
              onClick={fetchGateways}
              loading={loading}
            >
              Refresh
            </Button>
          </Space>
        </div>

        <Table
          columns={columns}
          dataSource={gateways}
          rowKey="id"
          loading={loading}
          pagination={{
            pageSize: 10,
            showSizeChanger: true,
            showQuickJumper: true,
            showTotal: (total, range) =>
              `${range[0]}-${range[1]} of ${total} gateways`,
          }}
          scroll={{ x: 1200 }}
          size="small"
          expandable={{
            expandedRowRender: (record) => (
              <Descriptions size="small" column={2}>
                <Descriptions.Item label="Capabilities">
                  <Space wrap>
                    {record.capabilities?.map((cap) => (
                      <Tag key={cap} color="blue">{cap}</Tag>
                    ))}
                  </Space>
                </Descriptions.Item>
                <Descriptions.Item label="Heartbeat Interval">
                  {record.config?.heartbeatInterval || '30s'}
                </Descriptions.Item>
                <Descriptions.Item label="Connection Timeout">
                  {record.config?.connectionTimeout || '10m'}
                </Descriptions.Item>
                <Descriptions.Item label="Enrollment Timeout">
                  {record.config?.enrollmentTimeout || '5m'}
                </Descriptions.Item>
                <Descriptions.Item label="Conditions" span={2}>
                  {record.conditions?.length > 0 ? (
                    <Space direction="vertical" size="small">
                      {record.conditions.map((condition, index) => (
                        <div key={index}>
                          <Tag color={condition.status === 'True' ? 'green' : 'red'}>
                            {condition.type}: {condition.status}
                          </Tag>
                          {condition.message && (
                            <div style={{ fontSize: '12px', color: '#666' }}>
                              {condition.message}
                            </div>
                          )}
                        </div>
                      ))}
                    </Space>
                  ) : 'No conditions'}
                </Descriptions.Item>
              </Descriptions>
            ),
            rowExpandable: (record) => record.capabilities?.length > 0 || record.conditions?.length > 0,
          }}
        />
      </Card>

      <Modal
        title="Create New Gateway"
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
          onFinish={handleCreateGateway}
        >
          <Form.Item
            name="name"
            label="Gateway Name"
            rules={[{ required: true, message: 'Please enter gateway name' }]}
          >
            <Input placeholder="Enter gateway name" />
          </Form.Item>

          <Form.Item
            name="endpoint"
            label="Endpoint"
            rules={[{ required: true, message: 'Please enter endpoint' }]}
          >
            <Input placeholder="gateway-1.wasmbed.svc.cluster.local:30430" />
          </Form.Item>

          <Form.Item
            name="capabilities"
            label="Capabilities"
            rules={[{ required: true, message: 'Please select capabilities' }]}
          >
            <Select
              mode="multiple"
              placeholder="Select capabilities"
            >
              <Option value="tls">TLS</Option>
              <Option value="wasm-deployment">WASM Deployment</Option>
              <Option value="device-management">Device Management</Option>
              <Option value="monitoring">Monitoring</Option>
            </Select>
          </Form.Item>

          <Form.Item>
            <Space>
              <Button type="primary" htmlType="submit">
                Create Gateway
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

      <Modal
        title={`Configure Gateway: ${selectedGateway?.name}`}
        open={configModalVisible}
        onCancel={() => {
          setConfigModalVisible(false);
          configForm.resetFields();
          setSelectedGateway(null);
        }}
        footer={null}
      >
        <Form
          form={configForm}
          layout="vertical"
          onFinish={handleUpdateGatewayConfig}
        >
          <Form.Item
            name="heartbeatInterval"
            label="Heartbeat Interval"
            rules={[{ required: true, message: 'Please enter heartbeat interval' }]}
          >
            <Select placeholder="Select heartbeat interval">
              <Option value="15s">15 seconds</Option>
              <Option value="30s">30 seconds</Option>
              <Option value="1m">1 minute</Option>
              <Option value="2m">2 minutes</Option>
              <Option value="5m">5 minutes</Option>
            </Select>
          </Form.Item>

          <Form.Item
            name="connectionTimeout"
            label="Connection Timeout"
            rules={[{ required: true, message: 'Please enter connection timeout' }]}
          >
            <Select placeholder="Select connection timeout">
              <Option value="5m">5 minutes</Option>
              <Option value="10m">10 minutes</Option>
              <Option value="15m">15 minutes</Option>
              <Option value="30m">30 minutes</Option>
              <Option value="1h">1 hour</Option>
            </Select>
          </Form.Item>

          <Form.Item
            name="enrollmentTimeout"
            label="Enrollment Timeout"
            rules={[{ required: true, message: 'Please enter enrollment timeout' }]}
          >
            <Select placeholder="Select enrollment timeout">
              <Option value="2m">2 minutes</Option>
              <Option value="5m">5 minutes</Option>
              <Option value="10m">10 minutes</Option>
              <Option value="15m">15 minutes</Option>
              <Option value="30m">30 minutes</Option>
            </Select>
          </Form.Item>

          <Form.Item>
            <Space>
              <Button type="primary" htmlType="submit">
                Update Configuration
              </Button>
              <Button onClick={() => {
                setConfigModalVisible(false);
                configForm.resetFields();
                setSelectedGateway(null);
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

export default GatewayManagement;
