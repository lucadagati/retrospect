/**
 * Mock data for dashboard when API is unavailable
 */

export const MOCK_GATEWAYS = [
  {
    id: "gateway-1",
    gateway_id: "gateway-1",
    name: "gateway-1",
    status: "Running",
    endpoint: "gateway-1-service.wasmbed.svc.cluster.local:8080",
    connected_devices: 3,
    enrolled_devices: 5,
    capabilities: ["TLS", "WASM", "OTA"],
    config: {
      heartbeatInterval: "30s",
      connectionTimeout: "10m",
      enrollmentTimeout: "5m"
    },
    conditions: [
      {
        type: "Ready",
        status: "True",
        message: "Gateway is running with 3 connected devices"
      }
    ],
    lastHeartbeat: new Date().toISOString()
  },
  {
    id: "gateway-2",
    gateway_id: "gateway-2",
    name: "gateway-2",
    status: "Running",
    endpoint: "gateway-2-service.wasmbed.svc.cluster.local:8080",
    connected_devices: 2,
    enrolled_devices: 4,
    capabilities: ["TLS", "WASM"],
    config: {
      heartbeatInterval: "30s",
      connectionTimeout: "10m",
      enrollmentTimeout: "5m"
    },
    conditions: [
      {
        type: "Ready",
        status: "True",
        message: "Gateway is running with 2 connected devices"
      }
    ],
    lastHeartbeat: new Date().toISOString()
  },
  {
    id: "gateway-3",
    gateway_id: "gateway-3",
    name: "gateway-3",
    status: "Pending",
    endpoint: "gateway-3-service.wasmbed.svc.cluster.local:8080",
    connected_devices: 0,
    enrolled_devices: 0,
    capabilities: ["TLS", "WASM", "OTA"],
    config: {
      heartbeatInterval: "30s",
      connectionTimeout: "10m",
      enrollmentTimeout: "5m"
    },
    conditions: [],
    lastHeartbeat: null
  }
];

export const MOCK_DEVICES = [
  {
    id: "device-1",
    device_id: "device-1",
    name: "device-1",
    type: "MCU",
    architecture: "ARM_CORTEX_M",
    mcuType: "RenodeArduinoNano33Ble",
    status: "Connected",
    gatewayId: "gateway-1",
    gateway_id: "gateway-1",
    gatewayName: "gateway-1",
    publicKey: "MCowBQYDK2VwAyEAYFG0s3FoisGisBrqNGZYY1RDCkhOqpe0jjF3oidNwuY=",
    lastHeartbeat: Math.floor(Date.now() / 1000),
    enrolled: true,
    connected: true,
    emulationStatus: "Running",
    renodeInstance: "renode-device-1",
    renodeEndpoint: "127.0.0.1:40483"
  },
  {
    id: "device-2",
    device_id: "device-2",
    name: "device-2",
    type: "MCU",
    architecture: "ARM_CORTEX_M",
    mcuType: "RenodeNrf52840",
    status: "Connected",
    gatewayId: "gateway-1",
    gateway_id: "gateway-1",
    gatewayName: "gateway-1",
    publicKey: "MCowBQYDK2VwAyEAXYZ1s4GpjtHjuCsrOHaZZ2SEDlkPrqf1kkG4pjeOwvZ=",
    lastHeartbeat: Math.floor(Date.now() / 1000),
    enrolled: true,
    connected: true,
    emulationStatus: "Running",
    renodeInstance: "renode-device-2",
    renodeEndpoint: "127.0.0.1:40484"
  },
  {
    id: "device-3",
    device_id: "device-3",
    name: "device-3",
    type: "MCU",
    architecture: "ARM_CORTEX_M",
    mcuType: "RenodeStm32f4",
    status: "Connected",
    gatewayId: "gateway-1",
    gateway_id: "gateway-1",
    gatewayName: "gateway-1",
    publicKey: "MCowBQYDK2VwAyEAZAB2t5HqkuIkvDtsP IbZa3TFEmkQsrg2llH5qkfPxwA=",
    lastHeartbeat: Math.floor(Date.now() / 1000),
    enrolled: true,
    connected: true,
    emulationStatus: "Running",
    renodeInstance: "renode-device-3",
    renodeEndpoint: "127.0.0.1:40485"
  },
  {
    id: "device-4",
    device_id: "device-4",
    name: "device-4",
    type: "MCU",
    architecture: "ARM_CORTEX_M",
    mcuType: "RenodeArduinoNano33Ble",
    status: "Enrolled",
    gatewayId: "gateway-2",
    gateway_id: "gateway-2",
    gatewayName: "gateway-2",
    publicKey: "MCowBQYDK2VwAyEAaBC3u6IrlvJlwEutQ JcZb4UGFnlRtsh3mmI6rlfQywB=",
    lastHeartbeat: Math.floor(Date.now() / 1000),
    enrolled: true,
    connected: false,
    emulationStatus: "Stopped",
    renodeInstance: null,
    renodeEndpoint: null
  },
  {
    id: "device-5",
    device_id: "device-5",
    name: "device-5",
    type: "MCU",
    architecture: "ARM_CORTEX_M",
    mcuType: "RenodeNrf52840",
    status: "Enrolled",
    gatewayId: "gateway-2",
    gateway_id: "gateway-2",
    gatewayName: "gateway-2",
    publicKey: "MCowBQYDK2VwAyEAbCD4v7JsmwKmxFvuR KdZc5VHGoMSuti4nnJ7smgRzxC=",
    lastHeartbeat: Math.floor(Date.now() / 1000),
    enrolled: true,
    connected: false,
    emulationStatus: "Stopped",
    renodeInstance: null,
    renodeEndpoint: null
  },
  {
    id: "device-6",
    device_id: "device-6",
    name: "device-6",
    type: "MCU",
    architecture: "ARM_CORTEX_M",
    mcuType: "RenodeStm32f4",
    status: "Pending",
    gatewayId: null,
    gateway_id: null,
    gatewayName: null,
    publicKey: null,
    lastHeartbeat: null,
    enrolled: false,
    connected: false,
    emulationStatus: "Stopped",
    renodeInstance: null,
    renodeEndpoint: null
  }
];

export const MOCK_APPLICATIONS = [
  {
    id: "app-1",
    app_id: "app-1",
    name: "Temperature Monitor",
    description: "Monitors device temperature and sends alerts",
    image: "wasmbed/temperature-monitor:v1.0",
    status: "Running",
    deployed_devices: ["device-1", "device-2", "device-3"],
    target_devices: ["device-1", "device-2", "device-3", "device-4"],
    created_at: Math.floor(Date.now() / 1000) - 86400 * 7,
    last_updated: Math.floor(Date.now() / 1000) - 3600,
    statistics: {
      total_devices: 4,
      target_count: 4,
      running_devices: 3,
      deployed_count: 3,
      failed_devices: 0,
      deployment_progress: 75.0
    }
  },
  {
    id: "app-2",
    app_id: "app-2",
    name: "LED Blinker",
    description: "Simple LED blinking application for testing",
    image: "wasmbed/led-blinker:v2.1",
    status: "Running",
    deployed_devices: ["device-1", "device-2"],
    target_devices: ["device-1", "device-2", "device-3"],
    created_at: Math.floor(Date.now() / 1000) - 86400 * 3,
    last_updated: Math.floor(Date.now() / 1000) - 7200,
    statistics: {
      total_devices: 3,
      target_count: 3,
      running_devices: 2,
      deployed_count: 2,
      failed_devices: 0,
      deployment_progress: 66.7
    }
  },
  {
    id: "app-3",
    app_id: "app-3",
    name: "Data Logger",
    description: "Logs sensor data to local storage",
    image: "wasmbed/data-logger:v1.5",
    status: "PartiallyRunning",
    deployed_devices: ["device-1"],
    target_devices: ["device-1", "device-2", "device-3", "device-4", "device-5"],
    created_at: Math.floor(Date.now() / 1000) - 86400 * 5,
    last_updated: Math.floor(Date.now() / 1000) - 1800,
    statistics: {
      total_devices: 5,
      target_count: 5,
      running_devices: 1,
      deployed_count: 1,
      failed_devices: 1,
      deployment_progress: 20.0
    }
  },
  {
    id: "app-4",
    app_id: "app-4",
    name: "Firmware Updater",
    description: "OTA firmware update manager",
    image: "wasmbed/firmware-updater:v3.0",
    status: "Pending",
    deployed_devices: [],
    target_devices: ["device-1", "device-2", "device-3", "device-4", "device-5", "device-6"],
    created_at: Math.floor(Date.now() / 1000) - 86400,
    last_updated: Math.floor(Date.now() / 1000) - 3600,
    statistics: {
      total_devices: 6,
      target_count: 6,
      running_devices: 0,
      deployed_count: 0,
      failed_devices: 0,
      deployment_progress: 0.0
    }
  },
  {
    id: "app-5",
    app_id: "app-5",
    name: "Network Diagnostics",
    description: "Network connectivity and latency monitoring",
    image: "wasmbed/network-diag:v1.2",
    status: "Failed",
    deployed_devices: [],
    target_devices: ["device-4", "device-5"],
    created_at: Math.floor(Date.now() / 1000) - 86400 * 2,
    last_updated: Math.floor(Date.now() / 1000) - 900,
    statistics: {
      total_devices: 2,
      target_count: 2,
      running_devices: 0,
      deployed_count: 0,
      failed_devices: 2,
      deployment_progress: 0.0
    }
  }
];

export const MOCK_SYSTEM_STATUS = {
  infrastructure: true,
  controllers: true,
  dashboard: true,
  gateways: MOCK_GATEWAYS.length,
  devices: MOCK_DEVICES.length
};

// Helper function to get mock data with simulated delay
export const getMockData = async (type, delay = 500) => {
  await new Promise(resolve => setTimeout(resolve, delay));
  
  switch(type) {
    case 'gateways':
      return { gateways: MOCK_GATEWAYS, success: true };
    case 'devices':
      return { devices: MOCK_DEVICES, success: true };
    case 'applications':
      return { applications: MOCK_APPLICATIONS, success: true };
    case 'status':
      return MOCK_SYSTEM_STATUS;
    default:
      return { success: false, message: 'Unknown mock data type' };
  }
};

// Helper to add random variation to mock data (for realism)
export const randomizeStatus = (items, statusField = 'status') => {
  const statuses = ['Running', 'Stopped', 'Pending', 'Failed'];
  return items.map(item => ({
    ...item,
    [statusField]: statuses[Math.floor(Math.random() * statuses.length)]
  }));
};

