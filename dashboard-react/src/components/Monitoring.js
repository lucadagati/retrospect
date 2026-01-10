import React, { useState, useEffect, useCallback } from 'react';
import {
  Card,
  Row,
  Col,
  Statistic,
  Typography,
  Table,
  Tag,
  Progress,
  Spin,
  Select,
  DatePicker,
  Space,
  Button,
} from 'antd';
import {
  CheckCircleOutlined,
  ExclamationCircleOutlined,
  ClockCircleOutlined,
  ReloadOutlined,
  LineChartOutlined,
  DatabaseOutlined,
  MobileOutlined,
  CloudServerOutlined,
} from '@ant-design/icons';
import dayjs from 'dayjs';
import { apiGet, apiAll, fetchWithTimeout } from '../utils/api';

const { Title } = Typography;
const { RangePicker } = DatePicker;
const { Option } = Select;

const Monitoring = () => {
  const [loading] = useState(false);
  const [metrics, setMetrics] = useState(null);
  const [logs, setLogs] = useState([]);
  const [timeRange, setTimeRange] = useState('1h');
  const [dateRange, setDateRange] = useState([
    dayjs().subtract(1, 'hour'),
    dayjs(),
  ]);

  const fetchMetrics = useCallback(async () => {
    try {
      // Fetch real data from backend APIs with timeout
      const [devicesData, applicationsData, gatewaysData, infraHealthResponse, infraStatusResponse] = await Promise.all([
        apiGet('/api/v1/devices', 10000),
        apiGet('/api/v1/applications', 10000),
        apiGet('/api/v1/gateways', 10000),
        fetchWithTimeout('/api/v1/infrastructure/health', {}, 5000),
        fetchWithTimeout('/api/v1/infrastructure/status', {}, 5000)
      ]);

      let devices = devicesData.devices || [];
      let applications = applicationsData.applications || [];
      let gateways = gatewaysData.gateways || [];
      
      // Determine infrastructure status from new endpoints
      let infraStatus = 'unknown';
      let infraComponents = {
        ca: 'unknown',
        secretStore: 'unknown',
        monitoring: 'unknown',
        logging: 'unknown'
      };

      if (infraHealthResponse.ok) {
        const infraHealthData = await infraHealthResponse.json();
        infraStatus = infraHealthData.status === 'healthy' ? 'active' : 'inactive';
      }

      if (infraStatusResponse.ok) {
        const infraStatusData = await infraStatusResponse.json();
        infraComponents = {
          ca: infraStatusData.components?.ca === 'healthy' ? 'active' : 'inactive',
          secretStore: infraStatusData.components?.secret_store === 'healthy' ? 'active' : 'inactive',
          monitoring: infraStatusData.components?.monitoring === 'healthy' ? 'active' : 'inactive',
          logging: infraStatusData.components?.logging === 'healthy' ? 'active' : 'inactive'
        };
      }

      // Calculate real metrics from API data
      const activeDevices = devices.filter(d => d.status === 'Connected').length;
      const enrolledDevices = devices.filter(d => d.enrolled).length;
      const runningApplications = applications.filter(a => a.status === 'Running').length;
      const activeGateways = gateways.filter(g => g.status === 'Active').length;
      const inactiveGateways = gateways.filter(g => g.status === 'Inactive').length;

      setMetrics({
        activeConnections: activeDevices,
        maxConnections: 30, // This would come from gateway config
        totalDevices: devices.length,
        activeDevices: activeDevices,
        totalApplications: applications.length,
        runningApplications: runningApplications,
        gatewayStatus: {
          active: activeGateways,
          inactive: inactiveGateways,
          totalDevices: activeDevices
        },
        infrastructureStatus: infraComponents,
        systemMetrics: {
          cpuUsage: 0, // Would need system metrics API
          memoryUsage: 0, // Would need system metrics API
          diskUsage: 0, // Would need system metrics API
          networkIn: 0, // Would need system metrics API
          networkOut: 0 // Would need system metrics API
        }
      });
    } catch (error) {
      console.error('Error fetching metrics:', error);
      // Set empty metrics when backend is not available
      setMetrics({
        activeConnections: 0,
        maxConnections: 0,
        totalDevices: 0,
        activeDevices: 0,
        totalApplications: 0,
        runningApplications: 0,
        gatewayStatus: {
          active: 0,
          inactive: 0,
          totalDevices: 0
        },
        infrastructureStatus: {
          ca: 'unknown',
          secretStore: 'unknown',
          monitoring: 'unknown',
          logging: 'unknown'
        },
        systemMetrics: {
          cpuUsage: 0,
          memoryUsage: 0,
          diskUsage: 0,
          networkIn: 0,
          networkOut: 0
        }
      });
    }
  }, []);

  const fetchLogs = useCallback(async () => {
    try {
      // Try to fetch logs from infrastructure service first, then fallback to general logs
      const [infraLogsResponse, generalLogsResponse] = await Promise.all([
        fetchWithTimeout('/api/v1/infrastructure/logs', {}, 5000).catch(() => ({ ok: false })),
        fetchWithTimeout('/api/v1/logs', {}, 5000).catch(() => ({ ok: false }))
      ]);
      
      let apiLogs = [];
      
      if (infraLogsResponse.ok) {
        try {
          const infraLogsData = await infraLogsResponse.json();
          apiLogs = infraLogsData.logs || [];
        } catch (e) {
          console.warn('Failed to parse infrastructure logs:', e);
        }
      }
      
      if (apiLogs.length === 0 && generalLogsResponse.ok) {
        try {
          const logsData = await generalLogsResponse.json();
          apiLogs = logsData.logs || [];
        } catch (e) {
          console.warn('Failed to parse general logs:', e);
        }
      }
      
      // Normalize log format (ensure all logs have required fields)
      const normalizedLogs = apiLogs.map((log, idx) => {
        // Handle both string and object formats
        if (typeof log === 'string') {
          return {
            id: idx + 1,
            timestamp: new Date().toISOString(),
            level: 'INFO',
            component: 'System',
            message: log
          };
        }
        
        // Ensure timestamp is ISO string
        let timestamp = log.timestamp;
        if (typeof timestamp === 'number') {
          timestamp = new Date(timestamp * 1000).toISOString();
        } else if (!timestamp) {
          timestamp = new Date().toISOString();
        }
        
        return {
          id: log.id || idx + 1,
          timestamp: timestamp,
          level: (log.level || 'INFO').toUpperCase(),
          component: log.component || 'System',
          message: log.message || log.toString()
        };
      });
      
      // If we have API logs, use them; otherwise generate system status logs
      if (normalizedLogs.length > 0) {
        setLogs(normalizedLogs);
      } else {
        // Generate system status logs as fallback
        try {
          const [devicesData, applicationsData, gatewaysData] = await apiAll([
            '/api/v1/devices',
            '/api/v1/applications',
            '/api/v1/gateways'
          ], 10000);

          let devices = devicesData.devices || [];
          let applications = applicationsData.applications || [];
          let gateways = gatewaysData.gateways || [];

          const systemLogs = [];
          
          if (gateways.length > 0) {
            systemLogs.push({
              id: 1,
              timestamp: new Date().toISOString(),
              level: 'INFO',
              component: 'Gateway',
              message: `${gateways.length} gateway(s) configured`
            });
          }

          if (devices.length > 0) {
            const connectedDevices = devices.filter(d => d.status === 'Connected').length;
            systemLogs.push({
              id: 2,
              timestamp: new Date(Date.now() - 30000).toISOString(),
              level: 'INFO',
              component: 'Device Controller',
              message: `${connectedDevices}/${devices.length} devices connected`
            });
          }

          if (applications.length > 0) {
            const runningApps = applications.filter(a => a.status === 'Running').length;
            systemLogs.push({
              id: 3,
              timestamp: new Date(Date.now() - 60000).toISOString(),
              level: 'INFO',
              component: 'Application Controller',
              message: `${runningApps}/${applications.length} applications running`
            });
          }

          setLogs(systemLogs);
        } catch (e) {
          console.error('Failed to generate system logs:', e);
          setLogs([]);
        }
      }
    } catch (error) {
      console.error('Error fetching logs:', error);
      // Set empty logs when backend is not available
      setLogs([]);
    }
  }, []);

  useEffect(() => {
    fetchMetrics();
    fetchLogs();
    const interval = setInterval(() => {
      fetchMetrics();
      fetchLogs();
    }, 30000); // Update every 30 seconds
    return () => clearInterval(interval);
  }, [fetchMetrics, fetchLogs]);

  const getLogLevelTag = (level) => {
    const levelConfig = {
      ERROR: { color: 'red', icon: <ExclamationCircleOutlined /> },
      WARN: { color: 'orange', icon: <ExclamationCircleOutlined /> },
      INFO: { color: 'blue', icon: <CheckCircleOutlined /> },
      DEBUG: { color: 'default', icon: <ClockCircleOutlined /> },
    };

    const config = levelConfig[level] || { color: 'default', icon: null };
    return (
      <Tag color={config.color} icon={config.icon}>
        {level}
      </Tag>
    );
  };

  const logColumns = [
    {
      title: 'Timestamp',
      dataIndex: 'timestamp',
      key: 'timestamp',
      render: (timestamp) => dayjs(timestamp).format('YYYY-MM-DD HH:mm:ss'),
      sorter: (a, b) => dayjs(a.timestamp).unix() - dayjs(b.timestamp).unix(),
    },
    {
      title: 'Level',
      dataIndex: 'level',
      key: 'level',
      render: (level) => getLogLevelTag(level),
      filters: [
        { text: 'ERROR', value: 'ERROR' },
        { text: 'WARN', value: 'WARN' },
        { text: 'INFO', value: 'INFO' },
        { text: 'DEBUG', value: 'DEBUG' },
      ],
      onFilter: (value, record) => record.level === value,
    },
    {
      title: 'Component',
      dataIndex: 'component',
      key: 'component',
      filters: [
        { text: 'Gateway', value: 'Gateway' },
        { text: 'Device Controller', value: 'Device Controller' },
        { text: 'Application Controller', value: 'Application Controller' },
        { text: 'Gateway Controller', value: 'Gateway Controller' },
        { text: 'Infrastructure', value: 'Infrastructure' },
      ],
      onFilter: (value, record) => record.component === value,
    },
    {
      title: 'Message',
      dataIndex: 'message',
      key: 'message',
      ellipsis: true,
    },
  ];

  if (loading) {
    return (
      <div style={{ textAlign: 'center', padding: '50px' }}>
        <Spin size="large" />
        <p>Loading monitoring data...</p>
      </div>
    );
  }

  return (
    <div>
      <Title level={2}>System Monitoring</Title>
      
      <div style={{ marginBottom: 24 }}>
        <Space>
          <Select
            value={timeRange}
            onChange={setTimeRange}
            style={{ width: 120 }}
          >
            <Option value="15m">Last 15m</Option>
            <Option value="1h">Last 1h</Option>
            <Option value="6h">Last 6h</Option>
            <Option value="24h">Last 24h</Option>
            <Option value="7d">Last 7d</Option>
          </Select>
          <RangePicker
            value={dateRange}
            onChange={setDateRange}
            showTime
            format="YYYY-MM-DD HH:mm:ss"
          />
          <Button
            icon={<ReloadOutlined />}
            onClick={() => {
              fetchMetrics();
              fetchLogs();
            }}
          >
            Refresh
          </Button>
        </Space>
      </div>

      {metrics && (
        <>
          <Row gutter={[16, 16]} style={{ marginBottom: 24 }}>
            <Col xs={24} sm={12} lg={6}>
              <Card>
                <Statistic
                  title="System Health"
                  value={metrics.systemHealth}
                  prefix={<CheckCircleOutlined />}
                  valueStyle={{ 
                    color: metrics.systemHealth === 'Good' ? '#3f8600' : '#cf1322' 
                  }}
                />
                <Progress
                  percent={metrics.systemHealth === 'Good' ? 100 : 60}
                  size="small"
                  status={metrics.systemHealth === 'Good' ? 'success' : 'exception'}
                />
              </Card>
            </Col>
            
            <Col xs={24} sm={12} lg={6}>
              <Card>
                <Statistic
                  title="Active Connections"
                  value={metrics.activeConnections}
                  prefix={<CloudServerOutlined />}
                  valueStyle={{ color: '#1890ff' }}
                />
                <div style={{ marginTop: 8, fontSize: '12px', color: '#666' }}>
                  Max: {metrics.maxConnections}
                </div>
              </Card>
            </Col>
            
            <Col xs={24} sm={12} lg={6}>
              <Card>
                <Statistic
                  title="CPU Usage"
                  value={metrics.cpuUsage}
                  suffix="%"
                  prefix={<LineChartOutlined />}
                  valueStyle={{ 
                    color: metrics.cpuUsage > 80 ? '#cf1322' : 
                           metrics.cpuUsage > 60 ? '#faad14' : '#3f8600' 
                  }}
                />
                <Progress
                  percent={metrics.cpuUsage}
                  size="small"
                  status={metrics.cpuUsage > 80 ? 'exception' : 'success'}
                />
              </Card>
            </Col>
            
            <Col xs={24} sm={12} lg={6}>
              <Card>
                <Statistic
                  title="Memory Usage"
                  value={metrics.memoryUsage}
                  suffix="%"
                  prefix={<DatabaseOutlined />}
                  valueStyle={{ 
                    color: metrics.memoryUsage > 80 ? '#cf1322' : 
                           metrics.memoryUsage > 60 ? '#faad14' : '#3f8600' 
                  }}
                />
                <Progress
                  percent={metrics.memoryUsage}
                  size="small"
                  status={metrics.memoryUsage > 80 ? 'exception' : 'success'}
                />
              </Card>
            </Col>
          </Row>

          <Row gutter={[16, 16]} style={{ marginBottom: 24 }}>
            <Col xs={24} lg={12}>
              <Card title="Device Status Distribution" size="small">
                <Row gutter={16}>
                  <Col span={12}>
                    <Statistic
                      title="Connected"
                      value={metrics.deviceStatus?.connected || 0}
                      valueStyle={{ color: '#3f8600' }}
                    />
                  </Col>
                  <Col span={12}>
                    <Statistic
                      title="Enrolled"
                      value={metrics.deviceStatus?.enrolled || 0}
                      valueStyle={{ color: '#1890ff' }}
                    />
                  </Col>
                </Row>
                <Row gutter={16} style={{ marginTop: 16 }}>
                  <Col span={12}>
                    <Statistic
                      title="Pending"
                      value={metrics.deviceStatus?.pending || 0}
                      valueStyle={{ color: '#faad14' }}
                    />
                  </Col>
                  <Col span={12}>
                    <Statistic
                      title="Failed"
                      value={metrics.deviceStatus?.failed || 0}
                      valueStyle={{ color: '#cf1322' }}
                    />
                  </Col>
                </Row>
              </Card>
            </Col>
            
            <Col xs={24} lg={12}>
              <Card title="Application Status Distribution" size="small">
                <Row gutter={16}>
                  <Col span={8}>
                    <Statistic
                      title="Running"
                      value={metrics.applicationStatus?.running || 0}
                      valueStyle={{ color: '#3f8600' }}
                    />
                  </Col>
                  <Col span={8}>
                    <Statistic
                      title="Deploying"
                      value={metrics.applicationStatus?.deploying || 0}
                      valueStyle={{ color: '#722ed1' }}
                    />
                  </Col>
                  <Col span={8}>
                    <Statistic
                      title="Failed"
                      value={metrics.applicationStatus?.failed || 0}
                      valueStyle={{ color: '#cf1322' }}
                    />
                  </Col>
                </Row>
              </Card>
            </Col>
          </Row>

          <Row gutter={[16, 16]} style={{ marginBottom: 24 }}>
            <Col xs={24} lg={12}>
              <Card title="Gateway Status" size="small">
                <Row gutter={16}>
                  <Col span={12}>
                    <Statistic
                      title="Active Gateways"
                      value={metrics.gatewayStatus?.active || 0}
                      valueStyle={{ color: '#3f8600' }}
                    />
                  </Col>
                  <Col span={12}>
                    <Statistic
                      title="Inactive Gateways"
                      value={metrics.gatewayStatus?.inactive || 0}
                      valueStyle={{ color: '#cf1322' }}
                    />
                  </Col>
                </Row>
                <div style={{ marginTop: 16 }}>
                  <Statistic
                    title="Total Devices Connected"
                    value={metrics.gatewayStatus?.totalDevices || 0}
                    prefix={<MobileOutlined />}
                    valueStyle={{ color: '#1890ff' }}
                  />
                </div>
              </Card>
            </Col>
            
            <Col xs={24} lg={12}>
              <Card title="Infrastructure Status" size="small">
                <Row gutter={16}>
                  <Col span={12}>
                    <Statistic
                      title="Certificate Authority"
                      value={metrics.infrastructureStatus?.ca === 'active' ? 'Active' : 'Inactive'}
                      valueStyle={{ 
                        color: metrics.infrastructureStatus?.ca === 'active' ? '#3f8600' : '#cf1322' 
                      }}
                    />
                  </Col>
                  <Col span={12}>
                    <Statistic
                      title="Secret Store"
                      value={metrics.infrastructureStatus?.secretStore === 'active' ? 'Active' : 'Inactive'}
                      valueStyle={{ 
                        color: metrics.infrastructureStatus?.secretStore === 'active' ? '#3f8600' : '#cf1322' 
                      }}
                    />
                  </Col>
                </Row>
                <Row gutter={16} style={{ marginTop: 16 }}>
                  <Col span={12}>
                    <Statistic
                      title="Monitoring"
                      value={metrics.infrastructureStatus?.monitoring === 'active' ? 'Active' : 'Inactive'}
                      valueStyle={{ 
                        color: metrics.infrastructureStatus?.monitoring === 'active' ? '#3f8600' : '#cf1322' 
                      }}
                    />
                  </Col>
                  <Col span={12}>
                    <Statistic
                      title="Logging"
                      value={metrics.infrastructureStatus?.logging === 'active' ? 'Active' : 'Inactive'}
                      valueStyle={{ 
                        color: metrics.infrastructureStatus?.logging === 'active' ? '#3f8600' : '#cf1322' 
                      }}
                    />
                  </Col>
                </Row>
              </Card>
            </Col>
          </Row>

          <Row gutter={[16, 16]} style={{ marginBottom: 24 }}>
            <Col xs={24}>
              <Card title="System Metrics" size="small">
                <Row gutter={16}>
                  <Col xs={12} sm={6}>
                    <Statistic
                      title="CPU Usage"
                      value={metrics.systemMetrics?.cpuUsage || 0}
                      suffix="%"
                      valueStyle={{ 
                        color: (metrics.systemMetrics?.cpuUsage || 0) > 80 ? '#cf1322' : 
                               (metrics.systemMetrics?.cpuUsage || 0) > 60 ? '#faad14' : '#3f8600'
                      }}
                    />
                    <Progress 
                      percent={metrics.systemMetrics?.cpuUsage || 0} 
                      size="small" 
                      status={(metrics.systemMetrics?.cpuUsage || 0) > 80 ? 'exception' : 'normal'}
                    />
                  </Col>
                  <Col xs={12} sm={6}>
                    <Statistic
                      title="Memory Usage"
                      value={metrics.systemMetrics?.memoryUsage || 0}
                      suffix="%"
                      valueStyle={{ 
                        color: (metrics.systemMetrics?.memoryUsage || 0) > 80 ? '#cf1322' : 
                               (metrics.systemMetrics?.memoryUsage || 0) > 60 ? '#faad14' : '#3f8600'
                      }}
                    />
                    <Progress 
                      percent={metrics.systemMetrics?.memoryUsage || 0} 
                      size="small" 
                      status={(metrics.systemMetrics?.memoryUsage || 0) > 80 ? 'exception' : 'normal'}
                    />
                  </Col>
                  <Col xs={12} sm={6}>
                    <Statistic
                      title="Disk Usage"
                      value={metrics.systemMetrics?.diskUsage || 0}
                      suffix="%"
                      valueStyle={{ 
                        color: (metrics.systemMetrics?.diskUsage || 0) > 80 ? '#cf1322' : 
                               (metrics.systemMetrics?.diskUsage || 0) > 60 ? '#faad14' : '#3f8600'
                      }}
                    />
                    <Progress 
                      percent={metrics.systemMetrics?.diskUsage || 0} 
                      size="small" 
                      status={(metrics.systemMetrics?.diskUsage || 0) > 80 ? 'exception' : 'normal'}
                    />
                  </Col>
                  <Col xs={12} sm={6}>
                    <Statistic
                      title="Network Traffic"
                      value={`${metrics.systemMetrics?.networkIn || 0} / ${metrics.systemMetrics?.networkOut || 0}`}
                      suffix="KB/s"
                      valueStyle={{ color: '#1890ff' }}
                    />
                  </Col>
                </Row>
              </Card>
            </Col>
          </Row>
        </>
      )}

      <Card title="System Logs" size="small">
        <Table
          columns={logColumns}
          dataSource={logs}
          rowKey={(record) => `log-${record.id || record.timestamp || Math.random()}`}
          size="small"
          pagination={{
            pageSize: 20,
            showSizeChanger: true,
            showQuickJumper: true,
            showTotal: (total, range) =>
              `${range[0]}-${range[1]} of ${total} logs`,
          }}
          scroll={{ y: 400 }}
        />
      </Card>
    </div>
  );
};

export default Monitoring;
