import React, { useState } from 'react';
import { 
  Box, 
  Drawer, 
  AppBar, 
  Toolbar, 
  Typography, 
  List, 
  ListItem, 
  ListItemIcon, 
  ListItemText, 
  IconButton,
  Badge,
  Chip,
  Divider,
  Avatar
} from '@mui/material';
import { 
  Dashboard as DashboardIcon,
  Flight as FlightIcon,
  Devices as DevicesIcon,
  Apps as AppsIcon,
  Monitor as MonitorIcon,
  Settings as SettingsIcon,
  Menu as MenuIcon,
  Notifications as NotificationsIcon,
  Wifi as WifiIcon,
  WifiOff as WifiOffIcon
} from '@mui/icons-material';

const drawerWidth = 260;

const DashboardLayout = ({ children, selectedTab, onTabChange, connectionStatus }) => {
  const [mobileOpen, setMobileOpen] = useState(false);

  const menuItems = [
    { text: 'Overview', icon: <DashboardIcon />, id: 'overview', badge: null },
    { text: 'Drone Control', icon: <FlightIcon />, id: 'drone', badge: null },
    { text: 'Devices', icon: <DevicesIcon />, id: 'devices', badge: 5 },
    { text: 'Applications', icon: <AppsIcon />, id: 'applications', badge: 8 },
    { text: 'Monitoring', icon: <MonitorIcon />, id: 'monitoring', badge: 2 },
    { text: 'Settings', icon: <SettingsIcon />, id: 'settings', badge: null },
  ];

  const handleDrawerToggle = () => {
    setMobileOpen(!mobileOpen);
  };

  const getConnectionIcon = () => {
    switch (connectionStatus) {
      case 'connected':
        return <WifiIcon color="success" />;
      case 'connecting':
        return <WifiIcon color="warning" className="pulse" />;
      default:
        return <WifiOffIcon color="error" />;
    }
  };

  const getConnectionColor = () => {
    switch (connectionStatus) {
      case 'connected': return 'success';
      case 'connecting': return 'warning';
      default: return 'error';
    }
  };

  const drawer = (
    <Box sx={{ height: '100%', display: 'flex', flexDirection: 'column' }}>
      <Toolbar sx={{ background: 'rgba(255,255,255,0.1)', mb: 1 }}>
        <Avatar sx={{ mr: 2, bgcolor: 'white', color: 'primary.main' }}>
          W
        </Avatar>
        <Box>
          <Typography variant="h6" noWrap component="div" sx={{ color: 'white', fontWeight: 600 }}>
            Wasmbed
          </Typography>
          <Typography variant="caption" sx={{ color: 'rgba(255,255,255,0.8)' }}>
            Platform v1.0
          </Typography>
        </Box>
      </Toolbar>
      
      <Box sx={{ px: 2, mb: 2 }}>
        <Chip
          icon={getConnectionIcon()}
          label={connectionStatus.charAt(0).toUpperCase() + connectionStatus.slice(1)}
          color={getConnectionColor()}
          size="small"
          variant="outlined"
          sx={{ 
            color: 'white', 
            borderColor: 'rgba(255,255,255,0.3)',
            '& .MuiChip-icon': { color: 'white' }
          }}
        />
      </Box>

      <Divider sx={{ borderColor: 'rgba(255,255,255,0.2)', mx: 2, mb: 1 }} />
      
      <List sx={{ flexGrow: 1, px: 1 }}>
        {menuItems.map((item) => (
          <ListItem 
            button 
            key={item.id}
            selected={selectedTab === item.id}
            onClick={() => onTabChange(item.id)}
            sx={{
              borderRadius: '12px',
              mb: 0.5,
              mx: 1,
              '&.Mui-selected': {
                backgroundColor: 'rgba(255,255,255,0.2)',
                '&:hover': {
                  backgroundColor: 'rgba(255,255,255,0.25)',
                },
                '& .MuiListItemIcon-root': {
                  color: 'white',
                },
                '& .MuiListItemText-primary': {
                  fontWeight: 600,
                }
              },
              '&:hover': {
                backgroundColor: 'rgba(255,255,255,0.1)',
              },
            }}
          >
            <ListItemIcon sx={{ color: 'rgba(255,255,255,0.8)', minWidth: 40 }}>
              {item.badge ? (
                <Badge badgeContent={item.badge} color="error" variant="dot">
                  {item.icon}
                </Badge>
              ) : (
                item.icon
              )}
            </ListItemIcon>
            <ListItemText 
              primary={item.text}
              primaryTypographyProps={{
                fontSize: '0.9rem',
                color: 'white'
              }}
            />
          </ListItem>
        ))}
      </List>

      <Box sx={{ p: 2, borderTop: '1px solid rgba(255,255,255,0.2)' }}>
        <Typography variant="caption" sx={{ color: 'rgba(255,255,255,0.6)' }}>
          © 2025 Wasmbed Platform
        </Typography>
      </Box>
    </Box>
  );

  return (
    <Box sx={{ display: 'flex', height: '100vh' }}>
      <AppBar
        position="fixed"
        sx={{
          width: { sm: `calc(100% - ${drawerWidth}px)` },
          ml: { sm: `${drawerWidth}px` },
          background: 'linear-gradient(90deg, #1976d2 0%, #1565c0 100%)',
          boxShadow: '0 2px 8px rgba(0,0,0,0.15)',
        }}
      >
        <Toolbar>
          <IconButton
            color="inherit"
            aria-label="open drawer"
            edge="start"
            onClick={handleDrawerToggle}
            sx={{ mr: 2, display: { sm: 'none' } }}
          >
            <MenuIcon />
          </IconButton>
          <Typography variant="h6" noWrap component="div" sx={{ flexGrow: 1, fontWeight: 600 }}>
            {menuItems.find(item => item.id === selectedTab)?.text || 'Dashboard'}
          </Typography>
          <IconButton color="inherit" sx={{ mr: 1 }}>
            <Badge badgeContent={2} color="error">
              <NotificationsIcon />
            </Badge>
          </IconButton>
          <Avatar sx={{ width: 32, height: 32, bgcolor: 'rgba(255,255,255,0.2)' }}>
            U
          </Avatar>
        </Toolbar>
      </AppBar>
      
      <Box
        component="nav"
        sx={{ width: { sm: drawerWidth }, flexShrink: { sm: 0 } }}
      >
        <Drawer
          variant="temporary"
          open={mobileOpen}
          onClose={handleDrawerToggle}
          ModalProps={{ keepMounted: true }}
          sx={{
            display: { xs: 'block', sm: 'none' },
            '& .MuiDrawer-paper': { 
              boxSizing: 'border-box', 
              width: drawerWidth,
              background: 'linear-gradient(180deg, #1976d2 0%, #1565c0 100%)',
            },
          }}
        >
          {drawer}
        </Drawer>
        <Drawer
          variant="permanent"
          sx={{
            display: { xs: 'none', sm: 'block' },
            '& .MuiDrawer-paper': { 
              boxSizing: 'border-box', 
              width: drawerWidth,
              background: 'linear-gradient(180deg, #1976d2 0%, #1565c0 100%)',
            },
          }}
          open
        >
          {drawer}
        </Drawer>
      </Box>
      
      <Box
        component="main"
        sx={{ 
          flexGrow: 1, 
          p: 3, 
          width: { sm: `calc(100% - ${drawerWidth}px)` },
          mt: 8,
          height: 'calc(100vh - 64px)',
          overflow: 'auto',
          backgroundColor: '#f5f5f5'
        }}
      >
        {children}
      </Box>
    </Box>
  );
};

export default DashboardLayout;