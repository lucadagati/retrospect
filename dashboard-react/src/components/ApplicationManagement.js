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
  Upload,
  message,
  Popconfirm,
  Typography,
  Row,
  Col,
  Statistic,
  Descriptions,
} from 'antd';
import {
  PlusOutlined,
  DeleteOutlined,
  ReloadOutlined,
  PlayCircleOutlined,
  PauseCircleOutlined,
  CheckCircleOutlined,
  ExclamationCircleOutlined,
  ClockCircleOutlined,
  UploadOutlined,
} from '@ant-design/icons';
import axios from 'axios';

const { Title } = Typography;
const { Option } = Select;
const { TextArea } = Input;

const ApplicationManagement = () => {
  const [applications, setApplications] = useState([]);
  const [devices, setDevices] = useState([]);
  const [loading, setLoading] = useState(false);
  const [modalVisible, setModalVisible] = useState(false);
  const [form] = Form.useForm();

  useEffect(() => {
    fetchApplications();
    fetchDevices();
  }, []);

  const fetchApplications = async () => {
    setLoading(true);
    try {
      const response = await axios.get('/api/applications');
      setApplications(response.data);
    } catch (error) {
      message.error('Failed to fetch applications');
      console.error('Error fetching applications:', error);
    } finally {
      setLoading(false);
    }
  };

  const fetchDevices = async () => {
    try {
      const response = await axios.get('/api/devices');
      setDevices(response.data);
    } catch (error) {
      console.error('Error fetching devices:', error);
    }
  };

  const handleCreateApplication = async (values) => {
    try {
      await axios.post('/api/applications', values);
      message.success('Application created successfully');
      setModalVisible(false);
      form.resetFields();
      fetchApplications();
    } catch (error) {
      message.error('Failed to create application');
      console.error('Error creating application:', error);
    }
  };

  const handleDeleteApplication = async (appId) => {
    try {
      await axios.delete(`/api/applications/${appId}`);
      message.success('Application deleted successfully');
      fetchApplications();
    } catch (error) {
      message.error('Failed to delete application');
      console.error('Error deleting application:', error);
    }
  };

  const handleDeployApplication = async (appId) => {
    try {
      await axios.post(`/api/applications/${appId}/deploy`);
      message.success('Application deployment started');
      fetchApplications();
    } catch (error) {
      message.error('Failed to deploy application');
      console.error('Error deploying application:', error);
    }
  };

  const handleStopApplication = async (appId) => {
    try {
      await axios.post(`/api/applications/${appId}/stop`);
      message.success('Application stopped');
      fetchApplications();
    } catch (error) {
      message.error('Failed to stop application');
      console.error('Error stopping application:', error);
    }
  };

  const getStatusTag = (status) => {
    const statusConfig = {
      Running: { color: 'green', icon: <CheckCircleOutlined /> },
      Deploying: { color: 'blue', icon: <ClockCircleOutlined /> },
      Pending: { color: 'orange', icon: <ClockCircleOutlined /> },
      Failed: { color: 'red', icon: <ExclamationCircleOutlined /> },
      Stopped: { color: 'default', icon: <PauseCircleOutlined /> },
      PartiallyRunning: { color: 'purple', icon: <ExclamationCircleOutlined /> },
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
        { text: 'Running', value: 'Running' },
        { text: 'Deploying', value: 'Deploying' },
        { text: 'Pending', value: 'Pending' },
        { text: 'Failed', value: 'Failed' },
        { text: 'Stopped', value: 'Stopped' },
        { text: 'PartiallyRunning', value: 'PartiallyRunning' },
      ],
      onFilter: (value, record) => record.status === value,
    },
    {
      title: 'Description',
      dataIndex: 'description',
      key: 'description',
      ellipsis: true,
    },
    {
      title: 'Target Devices',
      dataIndex: 'targetDevices',
      key: 'targetDevices',
      render: (targetDevices) => {
        if (targetDevices?.all_devices) {
          return <Tag color="blue">All Devices</Tag>;
        }
        if (targetDevices?.device_names?.length > 0) {
          return (
            <Space wrap>
              {targetDevices.device_names.slice(0, 2).map((name) => (
                <Tag key={name} color="blue">{name}</Tag>
              ))}
              {targetDevices.device_names.length > 2 && (
                <Tag color="blue">+{targetDevices.device_names.length - 2} more</Tag>
              )}
            </Space>
          );
        }
        return <Tag color="default">None</Tag>;
      },
    },
    {
      title: 'Statistics',
      key: 'statistics',
      render: (_, record) => {
        const stats = record.statistics;
        if (!stats) return '-';
        return (
          <Space direction="vertical" size="small">
            <div>Total: {stats.total_devices}</div>
            <div style={{ color: '#3f8600' }}>Running: {stats.running_devices}</div>
            <div style={{ color: '#cf1322' }}>Failed: {stats.failed_devices}</div>
          </Space>
        );
      },
    },
    {
      title: 'Last Updated',
      dataIndex: 'lastUpdated',
      key: 'lastUpdated',
      render: (timestamp) => timestamp ? new Date(timestamp).toLocaleString() : 'Never',
    },
    {
      title: 'Actions',
      key: 'actions',
      render: (_, record) => (
        <Space>
          {record.status === 'Pending' || record.status === 'Stopped' ? (
            <Button
              type="link"
              icon={<PlayCircleOutlined />}
              onClick={() => handleDeployApplication(record.id)}
            >
              Deploy
            </Button>
          ) : record.status === 'Running' ? (
            <Button
              type="link"
              icon={<PauseCircleOutlined />}
              onClick={() => handleStopApplication(record.id)}
            >
              Stop
            </Button>
          ) : null}
          <Popconfirm
            title="Are you sure you want to delete this application?"
            onConfirm={() => handleDeleteApplication(record.id)}
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

  const applicationStats = applications.reduce(
    (acc, app) => {
      acc.total++;
      if (app.status === 'Running') acc.running++;
      else if (app.status === 'Deploying') acc.deploying++;
      else if (app.status === 'Pending') acc.pending++;
      else if (app.status === 'Failed') acc.failed++;
      else if (app.status === 'Stopped') acc.stopped++;
      else if (app.status === 'PartiallyRunning') acc.partiallyRunning++;
      return acc;
    },
    { total: 0, running: 0, deploying: 0, pending: 0, failed: 0, stopped: 0, partiallyRunning: 0 }
  );

  return (
    <div>
      <Title level={2}>Application Management</Title>
      
      <Row gutter={[16, 16]} style={{ marginBottom: 24 }}>
        <Col xs={12} sm={6}>
          <Card size="small">
            <Statistic
              title="Total Applications"
              value={applicationStats.total}
              valueStyle={{ color: '#1890ff' }}
            />
          </Card>
        </Col>
        <Col xs={12} sm={6}>
          <Card size="small">
            <Statistic
              title="Running"
              value={applicationStats.running}
              valueStyle={{ color: '#3f8600' }}
            />
          </Card>
        </Col>
        <Col xs={12} sm={6}>
          <Card size="small">
            <Statistic
              title="Deploying"
              value={applicationStats.deploying}
              valueStyle={{ color: '#722ed1' }}
            />
          </Card>
        </Col>
        <Col xs={12} sm={6}>
          <Card size="small">
            <Statistic
              title="Failed"
              value={applicationStats.failed}
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
              Create Application
            </Button>
            <Button
              icon={<ReloadOutlined />}
              onClick={fetchApplications}
              loading={loading}
            >
              Refresh
            </Button>
          </Space>
        </div>

        <Table
          columns={columns}
          dataSource={applications}
          rowKey="id"
          loading={loading}
          pagination={{
            pageSize: 10,
            showSizeChanger: true,
            showQuickJumper: true,
            showTotal: (total, range) =>
              `${range[0]}-${range[1]} of ${total} applications`,
          }}
          expandable={{
            expandedRowRender: (record) => (
              <Descriptions size="small" column={2}>
                <Descriptions.Item label="Description">
                  {record.description || 'No description'}
                </Descriptions.Item>
                <Descriptions.Item label="Created At">
                  {record.createdAt ? new Date(record.createdAt).toLocaleString() : 'Unknown'}
                </Descriptions.Item>
                <Descriptions.Item label="Error" span={2}>
                  {record.error || 'No errors'}
                </Descriptions.Item>
              </Descriptions>
            ),
            rowExpandable: (record) => record.description || record.error,
          }}
        />
      </Card>

      <Modal
        title="Create New Application"
        open={modalVisible}
        onCancel={() => {
          setModalVisible(false);
          form.resetFields();
        }}
        footer={null}
        width={800}
      >
        <Form
          form={form}
          layout="vertical"
          onFinish={handleCreateApplication}
        >
          <Form.Item
            name="name"
            label="Application Name"
            rules={[{ required: true, message: 'Please enter application name' }]}
          >
            <Input placeholder="Enter application name" />
          </Form.Item>

          <Form.Item
            name="description"
            label="Description"
          >
            <TextArea
              rows={3}
              placeholder="Enter application description"
            />
          </Form.Item>

          <Form.Item
            name="targetDevices"
            label="Target Devices"
            rules={[{ required: true, message: 'Please select target devices' }]}
          >
            <Select
              mode="multiple"
              placeholder="Select target devices"
              optionFilterProp="children"
            >
              <Option value="all_devices">All Devices</Option>
              {devices.map((device) => (
                <Option key={device.id} value={device.name}>
                  {device.name} ({device.architecture})
                </Option>
              ))}
            </Select>
          </Form.Item>

          <Form.Item
            name="wasmBytes"
            label="WASM Binary"
            rules={[{ required: true, message: 'Please upload WASM binary' }]}
          >
            <Upload
              accept=".wasm"
              beforeUpload={(file) => {
                const reader = new FileReader();
                reader.onload = (e) => {
                  const arrayBuffer = e.target.result;
                  const uint8Array = new Uint8Array(arrayBuffer);
                  form.setFieldsValue({ wasmBytes: Array.from(uint8Array) });
                };
                reader.readAsArrayBuffer(file);
                return false; // Prevent upload
              }}
              showUploadList={false}
            >
              <Button icon={<UploadOutlined />}>
                Upload WASM Binary
              </Button>
            </Upload>
          </Form.Item>

          <Form.Item>
            <Space>
              <Button type="primary" htmlType="submit">
                Create Application
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

export default ApplicationManagement;
