import React, { useState } from 'react';
import { 
  Grid, 
  Card, 
  CardContent, 
  Typography, 
  Box, 
  Button,
  Chip,
  Table,
  TableBody,
  TableCell,
  TableContainer,
  TableHead,
  TableRow,
  Paper,
  IconButton,
  LinearProgress
} from '@mui/material';
import { 
  Apps as AppsIcon,
  PlayArrow as PlayIcon,
  Stop as StopIcon,
  Delete as DeleteIcon,
  CloudUpload as UploadIcon,
  Refresh as RefreshIcon
} from '@mui/icons-material';

const ApplicationsDashboard = () => {
  const [applications] = useState([
    {
      id: 'drone-control-v1',
      name: 'Drone Control System',
      device: 'drone-001',
      status: 'running',
      version: 'v1.2.3',
      deployedAt: '2025-01-10T10:30:00Z',
      memoryUsage: 2.5, // MB
      cpuUsage: 15, // %
      restarts: 0
    },
    {
      id: 'sensor-monitor',
      name: 'Sensor Monitor',
      device: 'drone-001',
      status: 'running',
      version: 'v0.8.1',
      deployedAt: '2025-01-10T09:15:00Z',
      memoryUsage: 1.2,
      cpuUsage: 8,
      restarts: 1
    },
    {
      id: 'navigation-assist',
      name: 'Navigation Assistant',
      device: 'drone-002',
      status: 'failed',
      version: 'v2.0.0',
      deployedAt: '2025-01-10T08:45:00Z',
      memoryUsage: 0,
      cpuUsage: 0,
      restarts: 3
    },
    {
      id: 'telemetry-relay',
      name: 'Telemetry Relay',
      device: 'gateway-001',
      status: 'deploying',
      version: 'v1.5.2',
      deployedAt: '2025-01-10T11:00:00Z',
      memoryUsage: 0.5,
      cpuUsage: 2,
      restarts: 0
    }
  ]);

  const getStatusColor = (status) => {
    switch (status) {
      case 'running': return 'success';
      case 'failed': return 'error';
      case 'deploying': return 'warning';
      case 'stopped': return 'default';
      default: return 'default';
    }
  };

  const formatDate = (dateString) => {
    return new Date(dateString).toLocaleString();
  };

  return (
    <Box>
      <Box display="flex" justifyContent="space-between" alignItems="center" mb={3}>
        <Typography variant="h4" gutterBottom>
          WASM Applications
        </Typography>
        <Box>
          <Button
            startIcon={<RefreshIcon />}
            onClick={() => window.location.reload()}
            sx={{ mr: 1 }}
          >
            Refresh
          </Button>
          <Button
            variant="contained"
            startIcon={<UploadIcon />}
          >
            Deploy App
          </Button>
        </Box>
      </Box>

      {/* Application Stats */}
      <Grid container spacing={3} sx={{ mb: 3 }}>
        <Grid item xs={12} sm={6} md={3}>
          <Card elevation={3}>
            <CardContent>
              <Box display="flex" alignItems="center" mb={2}>
                <AppsIcon color="primary" sx={{ mr: 1 }} />
                <Typography variant="h6">Total Apps</Typography>
              </Box>
              <Typography variant="h4" color="primary" fontWeight="bold">
                {applications.length}
              </Typography>
            </CardContent>
          </Card>
        </Grid>
        <Grid item xs={12} sm={6} md={3}>
          <Card elevation={3}>
            <CardContent>
              <Typography variant="h6" gutterBottom>Running</Typography>
              <Typography variant="h4" color="success.main" fontWeight="bold">
                {applications.filter(a => a.status === 'running').length}
              </Typography>
            </CardContent>
          </Card>
        </Grid>
        <Grid item xs={12} sm={6} md={3}>
          <Card elevation={3}>
            <CardContent>
              <Typography variant="h6" gutterBottom>Failed</Typography>
              <Typography variant="h4" color="error.main" fontWeight="bold">
                {applications.filter(a => a.status === 'failed').length}
              </Typography>
            </CardContent>
          </Card>
        </Grid>
        <Grid item xs={12} sm={6} md={3}>
          <Card elevation={3}>
            <CardContent>
              <Typography variant="h6" gutterBottom>Deploying</Typography>
              <Typography variant="h4" color="warning.main" fontWeight="bold">
                {applications.filter(a => a.status === 'deploying').length}
              </Typography>
            </CardContent>
          </Card>
        </Grid>
      </Grid>

      {/* Applications Table */}
      <Paper elevation={3}>
        <TableContainer>
          <Table>
            <TableHead>
              <TableRow>
                <TableCell>Application</TableCell>
                <TableCell>Device</TableCell>
                <TableCell>Status</TableCell>
                <TableCell>Version</TableCell>
                <TableCell>Memory</TableCell>
                <TableCell>CPU</TableCell>
                <TableCell>Deployed</TableCell>
                <TableCell>Restarts</TableCell>
                <TableCell>Actions</TableCell>
              </TableRow>
            </TableHead>
            <TableBody>
              {applications.map((app) => (
                <TableRow key={app.id} hover>
                  <TableCell>
                    <Box>
                      <Typography variant="subtitle2">
                        {app.name}
                      </Typography>
                      <Typography variant="caption" color="text.secondary">
                        {app.id}
                      </Typography>
                    </Box>
                  </TableCell>
                  <TableCell>{app.device}</TableCell>
                  <TableCell>
                    <Chip 
                      label={app.status}
                      color={getStatusColor(app.status)}
                      size="small"
                    />
                  </TableCell>
                  <TableCell>{app.version}</TableCell>
                  <TableCell>
                    <Box>
                      <Typography variant="body2">
                        {app.memoryUsage.toFixed(1)} MB
                      </Typography>
                      <LinearProgress 
                        variant="determinate" 
                        value={Math.min(app.memoryUsage * 10, 100)} 
                        size="small"
                        sx={{ width: 60, height: 4 }}
                      />
                    </Box>
                  </TableCell>
                  <TableCell>
                    <Box>
                      <Typography variant="body2">
                        {app.cpuUsage}%
                      </Typography>
                      <LinearProgress 
                        variant="determinate" 
                        value={app.cpuUsage} 
                        color={app.cpuUsage > 80 ? "error" : "primary"}
                        sx={{ width: 60, height: 4 }}
                      />
                    </Box>
                  </TableCell>
                  <TableCell>
                    <Typography variant="caption">
                      {formatDate(app.deployedAt)}
                    </Typography>
                  </TableCell>
                  <TableCell>
                    <Chip 
                      label={app.restarts}
                      color={app.restarts > 2 ? "error" : app.restarts > 0 ? "warning" : "default"}
                      size="small"
                    />
                  </TableCell>
                  <TableCell>
                    {app.status === 'running' ? (
                      <IconButton size="small" color="error" title="Stop">
                        <StopIcon />
                      </IconButton>
                    ) : (
                      <IconButton size="small" color="success" title="Start">
                        <PlayIcon />
                      </IconButton>
                    )}
                    <IconButton size="small" color="error" title="Delete">
                      <DeleteIcon />
                    </IconButton>
                  </TableCell>
                </TableRow>
              ))}
            </TableBody>
          </Table>
        </TableContainer>
      </Paper>
    </Box>
  );
};

export default ApplicationsDashboard;
