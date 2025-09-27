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
  Popconfirm,
  Typography,
  Row,
  Col,
  Statistic,
  Descriptions,
  Alert,
  Divider,
  Tooltip,
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
  RocketOutlined,
  CodeOutlined,
  InfoCircleOutlined,
  QuestionCircleOutlined,
} from '@ant-design/icons';
import GuidedDeployment from './GuidedDeployment';

const { Title } = Typography;
const { Option } = Select;
const { TextArea } = Input;

// Initial mock data
const initialApplications = [
  { id: 1, name: 'test-app-1', status: 'Pending', description: 'Test Application 1', targetDevices: ['mcu-board-1', 'mcu-board-2'] },
  { id: 2, name: 'test-app-2', status: 'Pending', description: 'Test Application 2', targetDevices: ['riscv-board-1', 'riscv-board-2'] }
];

const initialDevices = [
  { id: 1, name: 'mcu-board-1', status: 'Connected', type: 'MCU' },
  { id: 2, name: 'mcu-board-2', status: 'Connected', type: 'MCU' },
  { id: 3, name: 'mcu-board-3', status: 'Connected', type: 'MCU' },
  { id: 4, name: 'riscv-board-1', status: 'Connected', type: 'RISC-V' },
  { id: 5, name: 'riscv-board-2', status: 'Connected', type: 'RISC-V' },
  { id: 6, name: 'riscv-board-3', status: 'Connected', type: 'RISC-V' }
];

const ApplicationManagement = () => {
  const [applications, setApplications] = useState(initialApplications);
  const [devices, setDevices] = useState(initialDevices);
  const [loading, setLoading] = useState(false);
  const [modalVisible, setModalVisible] = useState(false);
  const [guidedDeploymentVisible, setGuidedDeploymentVisible] = useState(false);
  const [form] = Form.useForm();

  // Initialize data only once
  useEffect(() => {
    setLoading(false);
  }, []);

  const fetchApplications = async () => {
    setLoading(true);
    try {
      // In a real application, this would fetch from an API
      console.log('Refreshing applications list...');
    } catch (error) {
      console.error('Error fetching applications:', error);
    } finally {
      setLoading(false);
    }
  };

  const fetchDevices = async () => {
    try {
      // In a real application, this would fetch from an API
      console.log('Refreshing devices list...');
    } catch (error) {
      console.error('Error fetching devices:', error);
    }
  };

  const handleCreateApplication = async (values) => {
    try {
      // Create new application with unique ID
      const newApplication = {
        id: Date.now(), // Simple unique ID
        name: values.name,
        status: 'Pending',
        description: values.description,
        targetDevices: values.targetDevices || [],
        createdAt: new Date().toISOString()
      };
      
      // Add to applications list
      setApplications(prevApplications => [...prevApplications, newApplication]);
      
      console.log('Application created successfully:', newApplication);
      setModalVisible(false);
      form.resetFields();
    } catch (error) {
      console.error('Error creating application:', error);
    }
  };

  const handleDeleteApplication = async (appId) => {
    try {
      // Remove application from list
      setApplications(prevApplications => prevApplications.filter(app => app.id !== appId));
      console.log('Application deleted successfully:', appId);
    } catch (error) {
      console.error('Error deleting application:', error);
    }
  };

  const handleDeployApplication = async (appId) => {
    try {
      // Update application status to Deploying, then Running
      setApplications(prevApplications => 
        prevApplications.map(app => 
          app.id === appId 
            ? { ...app, status: 'Deploying' }
            : app
        )
      );
      
      // Simulate deployment process
      setTimeout(() => {
        setApplications(prevApplications => 
          prevApplications.map(app => 
            app.id === appId 
              ? { ...app, status: 'Running' }
              : app
          )
        );
      }, 2000);
      
      // Check if there are connected devices
      const connectedDevices = devices.filter(device => device.status === 'Connected');
      if (connectedDevices.length === 0) {
        console.log('No connected devices available for deployment');
        return;
      }
      
      console.log('Application deployment started:', appId);
    } catch (error) {
      console.error('Error deploying application:', error);
    }
  };

  const handleStopApplication = async (appId) => {
    try {
      // Update application status to Stopped
      setApplications(prevApplications => 
        prevApplications.map(app => 
          app.id === appId 
            ? { ...app, status: 'Stopped' }
            : app
        )
      );
      
      console.log('Application stopped:', appId);
    } catch (error) {
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
      width: 150,
      fixed: 'right',
      render: (_, record) => (
        <Space size="small">
          {record.status === 'Pending' || record.status === 'Stopped' ? (
            <Button
              type="link"
              icon={<PlayCircleOutlined />}
              onClick={() => handleDeployApplication(record.id)}
              size="small"
            >
              Deploy
            </Button>
          ) : record.status === 'Running' ? (
            <Button
              type="link"
              icon={<PauseCircleOutlined />}
              onClick={() => handleStopApplication(record.id)}
              size="small"
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
      
      {/* User Guidance Section */}
      <Alert
        message="Application Deployment Guide"
        description={
          <div>
            <p>Use the guided deployment wizard to easily compile and deploy your WASM applications:</p>
            <ul style={{ marginBottom: 0 }}>
              <li><strong>Step 1:</strong> Write or upload your source code (Rust, C/C++, AssemblyScript)</li>
              <li><strong>Step 2:</strong> Automatic compilation to WASM bytecode</li>
              <li><strong>Step 3:</strong> Select target devices and deploy</li>
              <li><strong>Step 4:</strong> Monitor deployment status</li>
            </ul>
          </div>
        }
        type="info"
        showIcon
        style={{ marginBottom: 24 }}
        action={
          <Button
            type="primary"
            icon={<RocketOutlined />}
            onClick={() => setGuidedDeploymentVisible(true)}
          >
            Start Guided Deployment
          </Button>
        }
      />
      
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
            <Tooltip title="Use the step-by-step wizard to compile and deploy WASM applications">
              <Button
                type="primary"
                icon={<RocketOutlined />}
                onClick={() => setGuidedDeploymentVisible(true)}
                size="large"
              >
                Guided Deployment
              </Button>
            </Tooltip>
            <Tooltip title="Quickly create a new application with basic settings">
              <Button
                icon={<PlusOutlined />}
                onClick={() => setModalVisible(true)}
              >
                Quick Create
              </Button>
            </Tooltip>
            <Tooltip title="Refresh the application list to get the latest status">
              <Button
                icon={<ReloadOutlined />}
                onClick={fetchApplications}
                loading={loading}
              >
                Refresh
              </Button>
            </Tooltip>
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
          scroll={{ x: 1200 }}
          size="small"
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

      <GuidedDeployment
        visible={guidedDeploymentVisible}
        onCancel={() => setGuidedDeploymentVisible(false)}
        onSuccess={(values) => {
          console.log('Application deployed successfully!', values);
          fetchApplications();
        }}
      />
    </div>
  );
};

export default ApplicationManagement;
