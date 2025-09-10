import React, { useState, useContext } from 'react';
import { 
  Grid, 
  Card, 
  CardContent, 
  Typography, 
  Box, 
  Table,
  TableBody,
  TableCell,
  TableContainer,
  TableHead,
  TableRow,
  Paper,
  Chip,
  IconButton,
  Button,
  Dialog,
  DialogTitle,
  DialogContent,
  DialogActions,
  TextField
} from '@mui/material';
import { 
  Devices as DevicesIcon,
  Add as AddIcon,
  Edit as EditIcon,
  Delete as DeleteIcon,
  Refresh as RefreshIcon,
  SignalWifi4Bar,
  SignalWifiOff
} from '@mui/icons-material';
import { WebSocketContext } from '../App';

const DevicesDashboard = () => {
  const { wsData } = useContext(WebSocketContext);
  const [devices] = useState([
    {
      id: 'drone-001',
      name: 'Primary Drone',
      type: 'Quadcopter',
      status: 'connected',
      lastSeen: '2 minutes ago',
      battery: 85,
      location: { x: 12.5, y: 8.3, z: 2.1 },
      firmware: 'v2.1.3'
    },
    {
      id: 'drone-002',
      name: 'Secondary Drone',
      type: 'Hexacopter',
      status: 'disconnected',
      lastSeen: '15 minutes ago',
      battery: 0,
      location: { x: 0, y: 0, z: 0 },
      firmware: 'v2.1.2'
    },
    {
      id: 'gateway-001',
      name: 'Main Gateway',
      type: 'Gateway MPU',
      status: 'connected',
      lastSeen: 'now',
      battery: null,
      location: null,
      firmware: 'v1.5.0'
    }
  ]);

  const [openDialog, setOpenDialog] = useState(false);

  const getStatusColor = (status) => {
    switch (status) {
      case 'connected': return 'success';
      case 'disconnected': return 'error';
      case 'enrolling': return 'warning';
      default: return 'default';
    }
  };

  const getStatusIcon = (status) => {
    switch (status) {
      case 'connected': return <SignalWifi4Bar color="success" />;
      default: return <SignalWifiOff color="error" />;
    }
  };

  return (
    <Box>
      <Box display="flex" justifyContent="space-between" alignItems="center" mb={3}>
        <Typography variant="h4" gutterBottom>
          Device Management
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
            startIcon={<AddIcon />}
            onClick={() => setOpenDialog(true)}
          >
            Add Device
          </Button>
        </Box>
      </Box>

      {/* Device Stats */}
      <Grid container spacing={3} sx={{ mb: 3 }}>
        <Grid item xs={12} sm={6} md={3}>
          <Card elevation={3}>
            <CardContent>
              <Box display="flex" alignItems="center" mb={2}>
                <DevicesIcon color="primary" sx={{ mr: 1 }} />
                <Typography variant="h6">Total Devices</Typography>
              </Box>
              <Typography variant="h4" color="primary" fontWeight="bold">
                {devices.length}
              </Typography>
            </CardContent>
          </Card>
        </Grid>
        <Grid item xs={12} sm={6} md={3}>
          <Card elevation={3}>
            <CardContent>
              <Typography variant="h6" gutterBottom>Connected</Typography>
              <Typography variant="h4" color="success.main" fontWeight="bold">
                {devices.filter(d => d.status === 'connected').length}
              </Typography>
            </CardContent>
          </Card>
        </Grid>
        <Grid item xs={12} sm={6} md={3}>
          <Card elevation={3}>
            <CardContent>
              <Typography variant="h6" gutterBottom>Offline</Typography>
              <Typography variant="h4" color="error.main" fontWeight="bold">
                {devices.filter(d => d.status === 'disconnected').length}
              </Typography>
            </CardContent>
          </Card>
        </Grid>
        <Grid item xs={12} sm={6} md={3}>
          <Card elevation={3}>
            <CardContent>
              <Typography variant="h6" gutterBottom>Enrolling</Typography>
              <Typography variant="h4" color="warning.main" fontWeight="bold">
                {devices.filter(d => d.status === 'enrolling').length}
              </Typography>
            </CardContent>
          </Card>
        </Grid>
      </Grid>

      {/* Device Table */}
      <Paper elevation={3}>
        <TableContainer>
          <Table>
            <TableHead>
              <TableRow>
                <TableCell>Device</TableCell>
                <TableCell>Type</TableCell>
                <TableCell>Status</TableCell>
                <TableCell>Last Seen</TableCell>
                <TableCell>Battery</TableCell>
                <TableCell>Location</TableCell>
                <TableCell>Firmware</TableCell>
                <TableCell>Actions</TableCell>
              </TableRow>
            </TableHead>
            <TableBody>
              {devices.map((device) => (
                <TableRow key={device.id} hover>
                  <TableCell>
                    <Box display="flex" alignItems="center">
                      {getStatusIcon(device.status)}
                      <Box ml={2}>
                        <Typography variant="subtitle2">
                          {device.name}
                        </Typography>
                        <Typography variant="caption" color="text.secondary">
                          {device.id}
                        </Typography>
                      </Box>
                    </Box>
                  </TableCell>
                  <TableCell>{device.type}</TableCell>
                  <TableCell>
                    <Chip 
                      label={device.status}
                      color={getStatusColor(device.status)}
                      size="small"
                    />
                  </TableCell>
                  <TableCell>{device.lastSeen}</TableCell>
                  <TableCell>
                    {device.battery !== null ? `${device.battery}%` : 'N/A'}
                  </TableCell>
                  <TableCell>
                    {device.location 
                      ? `${device.location.x.toFixed(1)}, ${device.location.y.toFixed(1)}, ${device.location.z.toFixed(1)}`
                      : 'N/A'
                    }
                  </TableCell>
                  <TableCell>{device.firmware}</TableCell>
                  <TableCell>
                    <IconButton size="small" color="primary">
                      <EditIcon />
                    </IconButton>
                    <IconButton size="small" color="error">
                      <DeleteIcon />
                    </IconButton>
                  </TableCell>
                </TableRow>
              ))}
            </TableBody>
          </Table>
        </TableContainer>
      </Paper>

      {/* Add Device Dialog */}
      <Dialog open={openDialog} onClose={() => setOpenDialog(false)} maxWidth="sm" fullWidth>
        <DialogTitle>Add New Device</DialogTitle>
        <DialogContent>
          <TextField
            autoFocus
            margin="dense"
            label="Device Name"
            fullWidth
            variant="outlined"
            sx={{ mb: 2 }}
          />
          <TextField
            margin="dense"
            label="Device Type"
            fullWidth
            variant="outlined"
            sx={{ mb: 2 }}
          />
          <TextField
            margin="dense"
            label="Public Key"
            fullWidth
            variant="outlined"
            multiline
            rows={3}
          />
        </DialogContent>
        <DialogActions>
          <Button onClick={() => setOpenDialog(false)}>Cancel</Button>
          <Button variant="contained" onClick={() => setOpenDialog(false)}>Add Device</Button>
        </DialogActions>
      </Dialog>
    </Box>
  );
};

export default DevicesDashboard;
