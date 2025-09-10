import React, { useState, useEffect, useContext } from 'react';
import { 
  Grid, 
  Card, 
  CardContent, 
  Typography, 
  Box, 
  Chip, 
  LinearProgress, 
  List, 
  ListItem, 
  ListItemText, 
  ListItemIcon,
  Alert, 
  AlertTitle,
  Paper,
  CircularProgress,
  Divider
} from '@mui/material';
import { 
  CheckCircle, 
  Error, 
  Warning, 
  Info,
  Devices, 
  Apps, 
  Storage, 
  Speed,
  Memory,
  Computer,
  TrendingUp,
  TrendingDown,
  Timeline
} from '@mui/icons-material';
import { 
  LineChart, 
  Line, 
  XAxis, 
  YAxis, 
  CartesianGrid, 
  Tooltip, 
  ResponsiveContainer,
  AreaChart,
  Area,
  PieChart,
  Pie,
  Cell
} from 'recharts';
import { WebSocketContext } from '../App';

const COLORS = ['#0088FE', '#00C49F', '#FFBB28', '#FF8042'];

const StatCard = ({ title, value, subtitle, icon, color, trend, loading }) => (
  <Card elevation={3} sx={{ height: '100%' }}>
    <CardContent>
      <Box display="flex" alignItems="center" justifyContent="space-between" mb={2}>
        <Box display="flex" alignItems="center">
          <Box sx={{ color: `${color}.main`, mr: 1 }}>
            {icon}
          </Box>
          <Typography variant="h6" color="text.secondary">
            {title}
          </Typography>
        </Box>
        {trend && (
          <Box display="flex" alignItems="center">
            {trend > 0 ? (
              <TrendingUp color="success" fontSize="small" />
            ) : (
              <TrendingDown color="error" fontSize="small" />
            )}
            <Typography variant="caption" color={trend > 0 ? 'success.main' : 'error.main'}>
              {Math.abs(trend)}%
            </Typography>
          </Box>
        )}
      </Box>
      
      {loading ? (
        <CircularProgress size={24} />
      ) : (
        <Typography variant="h4" color={`${color}.main`} fontWeight="bold" gutterBottom>
          {value}
        </Typography>
      )}
      
      {subtitle && (
        <Typography variant="body2" color="text.secondary">
          {subtitle}
        </Typography>
      )}
    </CardContent>
  </Card>
);

const OverviewDashboard = () => {
  const { wsData, connectionStatus } = useContext(WebSocketContext);
  const [systemStatus, setSystemStatus] = useState({
    pods: { total: 12, running: 10, pending: 1, failed: 1 },
    devices: { total: 5, connected: 3, disconnected: 1, enrolling: 1 },
    applications: { total: 8, running: 6, failed: 1, deploying: 1 },
    resources: { cpu: 45, memory: 67, storage: 23, network: 85 }
  });

  const [alerts, setAlerts] = useState([
    {
      severity: 'warning',
      title: 'High CPU Usage',
      message: 'CPU usage on gateway-pod-1 is above 80%',
      timestamp: '2 minutes ago'
    },
    {
      severity: 'error',
      title: 'Device Disconnected',
      message: 'Device drone-002 has been disconnected',
      timestamp: '5 minutes ago'
    }
  ]);

  const [metrics, setMetrics] = useState([]);
  const [pieData, setPieData] = useState([
    { name: 'Running', value: 10, color: '#4caf50' },
    { name: 'Pending', value: 1, color: '#ff9800' },
    { name: 'Failed', value: 1, color: '#f44336' },
  ]);

  useEffect(() => {
    // Genera dati metriche storiche
    const generateMetrics = () => {
      const now = new Date();
      const data = [];
      for (let i = 23; i >= 0; i--) {
        const time = new Date(now.getTime() - i * 60 * 60 * 1000);
        data.push({
          time: time.toLocaleTimeString('it-IT', { hour: '2-digit', minute: '2-digit' }),
          cpu: 20 + Math.random() * 60,
          memory: 30 + Math.random() * 50,
          devices: Math.floor(Math.random() * 5) + 3,
          applications: Math.floor(Math.random() * 8) + 5,
          network: 10 + Math.random() * 80
        });
      }
      setMetrics(data);
    };

    generateMetrics();
    const interval = setInterval(generateMetrics, 30000); // Aggiorna ogni 30 secondi

    return () => clearInterval(interval);
  }, []);

  // Aggiorna le risorse in tempo reale
  useEffect(() => {
    if (wsData.metrics && wsData.metrics.cpu) {
      setSystemStatus(prev => ({
        ...prev,
        resources: {
          ...prev.resources,
          cpu: Math.round(wsData.metrics.cpu),
          memory: Math.round(wsData.metrics.memory || prev.resources.memory),
        }
      }));
    }
  }, [wsData.metrics]);

  const getStatusIcon = (status) => {
    switch (status) {
      case 'running': case 'connected': return <CheckCircle color="success" />;
      case 'pending': case 'enrolling': case 'deploying': return <Warning color="warning" />;
      case 'failed': case 'disconnected': return <Error color="error" />;
      case 'warning': return <Warning color="warning" />;
      default: return <Info color="info" />;
    }
  };

  const recentActivity = [
    { type: 'connected', message: 'Device drone-001 connected', timestamp: '2 minutes ago' },
    { type: 'running', message: 'Application drone-control deployed', timestamp: '5 minutes ago' },
    { type: 'warning', message: 'High CPU usage detected', timestamp: '10 minutes ago' },
    { type: 'running', message: 'Pod wasmbed-gateway-2 started', timestamp: '15 minutes ago' },
  ];

  return (
    <Box>
      {/* Status Cards */}
      <Grid container spacing={3} sx={{ mb: 3 }}>
        <Grid item xs={12} sm={6} md={3}>
          <StatCard
            title="Devices"
            value={systemStatus.devices.total}
            subtitle={`${systemStatus.devices.connected} connected, ${systemStatus.devices.disconnected} offline`}
            icon={<Devices />}
            color="primary"
            trend={2.5}
            loading={connectionStatus === 'connecting'}
          />
        </Grid>

        <Grid item xs={12} sm={6} md={3}>
          <StatCard
            title="Applications"
            value={systemStatus.applications.total}
            subtitle={`${systemStatus.applications.running} running, ${systemStatus.applications.failed} failed`}
            icon={<Apps />}
            color="secondary"
            trend={-1.2}
          />
        </Grid>

        <Grid item xs={12} sm={6} md={3}>
          <StatCard
            title="Kubernetes Pods"
            value={systemStatus.pods.total}
            subtitle={`${systemStatus.pods.running} running, ${systemStatus.pods.failed} failed`}
            icon={<Storage />}
            color="success"
            trend={0.8}
          />
        </Grid>

        <Grid item xs={12} sm={6} md={3}>
          <StatCard
            title="System Health"
            value="98.5%"
            subtitle="All systems operational"
            icon={<Speed />}
            color="info"
            trend={0.3}
          />
        </Grid>
      </Grid>

      {/* Resource Usage */}
      <Grid container spacing={3} sx={{ mb: 3 }}>
        <Grid item xs={12} md={8}>
          <Paper elevation={3} sx={{ p: 3, height: '100%' }}>
            <Typography variant="h6" gutterBottom display="flex" alignItems="center">
              <Timeline sx={{ mr: 1 }} />
              System Metrics (24h)
            </Typography>
            <ResponsiveContainer width="100%" height={300}>
              <AreaChart data={metrics}>
                <defs>
                  <linearGradient id="colorCpu" x1="0" y1="0" x2="0" y2="1">
                    <stop offset="5%" stopColor="#1976d2" stopOpacity={0.8}/>
                    <stop offset="95%" stopColor="#1976d2" stopOpacity={0}/>
                  </linearGradient>
                  <linearGradient id="colorMemory" x1="0" y1="0" x2="0" y2="1">
                    <stop offset="5%" stopColor="#dc004e" stopOpacity={0.8}/>
                    <stop offset="95%" stopColor="#dc004e" stopOpacity={0}/>
                  </linearGradient>
                </defs>
                <CartesianGrid strokeDasharray="3 3" />
                <XAxis dataKey="time" />
                <YAxis />
                <Tooltip />
                <Area 
                  type="monotone" 
                  dataKey="cpu" 
                  stroke="#1976d2" 
                  fillOpacity={1} 
                  fill="url(#colorCpu)" 
                  strokeWidth={2}
                />
                <Area 
                  type="monotone" 
                  dataKey="memory" 
                  stroke="#dc004e" 
                  fillOpacity={1} 
                  fill="url(#colorMemory)" 
                  strokeWidth={2}
                />
              </AreaChart>
            </ResponsiveContainer>
          </Paper>
        </Grid>

        <Grid item xs={12} md={4}>
          <Paper elevation={3} sx={{ p: 3, height: '100%' }}>
            <Typography variant="h6" gutterBottom>
              Resource Usage
            </Typography>
            
            <Box mb={3}>
              <Box display="flex" alignItems="center" mb={1}>
                <Computer sx={{ mr: 1, fontSize: 20 }} />
                <Typography variant="body2" sx={{ minWidth: '80px' }}>
                  CPU: {systemStatus.resources.cpu}%
                </Typography>
              </Box>
              <LinearProgress 
                variant="determinate" 
                value={systemStatus.resources.cpu} 
                color={systemStatus.resources.cpu > 80 ? "error" : "primary"}
                sx={{ height: 8, borderRadius: 4 }}
              />
            </Box>

            <Box mb={3}>
              <Box display="flex" alignItems="center" mb={1}>
                <Memory sx={{ mr: 1, fontSize: 20 }} />
                <Typography variant="body2" sx={{ minWidth: '80px' }}>
                  Memory: {systemStatus.resources.memory}%
                </Typography>
              </Box>
              <LinearProgress 
                variant="determinate" 
                value={systemStatus.resources.memory} 
                color={systemStatus.resources.memory > 80 ? "error" : "secondary"}
                sx={{ height: 8, borderRadius: 4 }}
              />
            </Box>

            <Box mb={3}>
              <Box display="flex" alignItems="center" mb={1}>
                <Storage sx={{ mr: 1, fontSize: 20 }} />
                <Typography variant="body2" sx={{ minWidth: '80px' }}>
                  Storage: {systemStatus.resources.storage}%
                </Typography>
              </Box>
              <LinearProgress 
                variant="determinate" 
                value={systemStatus.resources.storage} 
                color="success"
                sx={{ height: 8, borderRadius: 4 }}
              />
            </Box>

            <Divider sx={{ my: 2 }} />
            
            <Typography variant="subtitle2" gutterBottom>
              Pod Distribution
            </Typography>
            <ResponsiveContainer width="100%" height={120}>
              <PieChart>
                <Pie
                  data={pieData}
                  cx="50%"
                  cy="50%"
                  innerRadius={30}
                  outerRadius={50}
                  paddingAngle={5}
                  dataKey="value"
                >
                  {pieData.map((entry, index) => (
                    <Cell key={`cell-${index}`} fill={entry.color} />
                  ))}
                </Pie>
                <Tooltip />
              </PieChart>
            </ResponsiveContainer>
          </Paper>
        </Grid>
      </Grid>

      {/* Alerts and Activity */}
      <Grid container spacing={3}>
        <Grid item xs={12} md={6}>
          {alerts.length > 0 && (
            <Paper elevation={3} sx={{ p: 3, height: '100%' }}>
              <Typography variant="h6" gutterBottom>
                Active Alerts
              </Typography>
              {alerts.map((alert, index) => (
                <Alert 
                  key={index} 
                  severity={alert.severity} 
                  sx={{ mb: 1 }}
                >
                  <AlertTitle>{alert.title}</AlertTitle>
                  {alert.message}
                  <Typography variant="caption" display="block" sx={{ mt: 1 }}>
                    {alert.timestamp}
                  </Typography>
                </Alert>
              ))}
            </Paper>
          )}
        </Grid>

        <Grid item xs={12} md={6}>
          <Paper elevation={3} sx={{ p: 3, height: '100%' }}>
            <Typography variant="h6" gutterBottom>
              Recent Activity
            </Typography>
            <List sx={{ maxHeight: 300, overflow: 'auto' }}>
              {recentActivity.map((activity, index) => (
                <ListItem key={index} divider={index < recentActivity.length - 1}>
                  <ListItemIcon>
                    {getStatusIcon(activity.type)}
                  </ListItemIcon>
                  <ListItemText 
                    primary={activity.message} 
                    secondary={activity.timestamp}
                    primaryTypographyProps={{ variant: 'body2' }}
                    secondaryTypographyProps={{ variant: 'caption' }}
                  />
                </ListItem>
              ))}
            </List>
          </Paper>
        </Grid>
      </Grid>
    </Box>
  );
};

export default OverviewDashboard;