# Wasmbed Dashboard React

A modern React-based dashboard for the Wasmbed Platform, providing comprehensive management and monitoring capabilities.

## Features

- **Dashboard Overview**: System status, device counts, application metrics
- **Device Management**: Create, list, and manage edge devices
- **Application Management**: Deploy and monitor WASM applications
- **Gateway Management**: Monitor gateway status and connections
- **Monitoring**: Real-time metrics, logs, and system health
- **Responsive Design**: Works on desktop and mobile devices

## Technology Stack

- **React 18**: Modern React with hooks
- **Ant Design 5**: Professional UI components
- **React Router 6**: Client-side routing
- **Axios**: HTTP client for API calls
- **Day.js**: Date manipulation library

## Getting Started

### Prerequisites

- Node.js 16+ 
- npm or yarn
- Wasmbed Platform running on localhost:30470

### Installation

1. Install dependencies:
```bash
npm install
```

2. Start the development server:
```bash
npm start
```

3. Open [http://localhost:3000](http://localhost:3000) in your browser

### Building for Production

```bash
npm run build
```

This creates a `build` folder with optimized production files.

## API Integration

The dashboard connects to the Wasmbed Platform services:

- **Gateway API**: http://localhost:30451
- **Infrastructure API**: http://localhost:30460
- **Dashboard Service**: http://localhost:30470

## Components

### Dashboard
- System overview with key metrics
- Quick actions for common tasks
- System health indicators

### Device Management
- Device listing and status
- Device creation and configuration
- Connection monitoring

### Application Management
- Application deployment
- Status monitoring
- Configuration management

### Gateway Management
- Gateway status monitoring
- Connection management
- Performance metrics

### Monitoring
- Real-time system metrics
- Log viewing and filtering
- Performance charts
- System health indicators

## Development

### Project Structure

```
src/
├── components/
│   ├── Dashboard.js          # Main dashboard overview
│   ├── DeviceManagement.js   # Device management interface
│   ├── ApplicationManagement.js # Application management
│   ├── GatewayManagement.js  # Gateway monitoring
│   └── Monitoring.js         # System monitoring
├── App.js                    # Main app component
└── index.js                  # Entry point
```

### Mock Data

The dashboard includes mock data for development when backend services are not available. This allows for:

- UI development without backend dependencies
- Demonstration of functionality
- Testing of error handling

### Styling

The dashboard uses Ant Design's design system with:

- Consistent color scheme
- Responsive grid layout
- Professional typography
- Accessible components

## Deployment

### Static Hosting

The built files can be served by any static file server:

```bash
# Using serve
npx serve -s build

# Using nginx
# Copy build/ contents to nginx html directory
```

### Docker

```dockerfile
FROM nginx:alpine
COPY build/ /usr/share/nginx/html/
EXPOSE 80
CMD ["nginx", "-g", "daemon off;"]
```

## Configuration

### Environment Variables

Create a `.env` file for configuration:

```env
REACT_APP_API_BASE_URL=http://localhost:30470
REACT_APP_GATEWAY_URL=http://localhost:30451
REACT_APP_INFRASTRUCTURE_URL=http://localhost:30460
```

### Proxy Configuration

The dashboard uses a proxy configuration in `package.json` to connect to the Wasmbed Platform services.

## Contributing

1. Follow React best practices
2. Use Ant Design components consistently
3. Add proper error handling
4. Include loading states
5. Write responsive designs
6. Test on multiple screen sizes

## License

AGPL-3.0 - See LICENSE file for details
