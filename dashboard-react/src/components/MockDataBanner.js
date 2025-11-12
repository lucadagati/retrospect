import React, { useState, useEffect } from 'react';
import { Alert, Space, Typography } from 'antd';
import { InfoCircleOutlined, DatabaseOutlined } from '@ant-design/icons';

const { Text } = Typography;

const MockDataBanner = () => {
  const [showBanner, setShowBanner] = useState(false);
  const [mockUsageCount, setMockUsageCount] = useState(0);

  useEffect(() => {
    // Listen for console logs about mock data usage
    const originalWarn = console.warn;
    const originalLog = console.log;

    console.warn = function(...args) {
      if (args[0] && args[0].includes('mock fallback')) {
        setShowBanner(true);
        setMockUsageCount(prev => prev + 1);
      }
      originalWarn.apply(console, args);
    };

    console.log = function(...args) {
      if (args[0] && args[0].includes('Using mock')) {
        setShowBanner(true);
        setMockUsageCount(prev => prev + 1);
      }
      originalLog.apply(console, args);
    };

    return () => {
      console.warn = originalWarn;
      console.log = originalLog;
    };
  }, []);

  if (!showBanner) {
    return null;
  }

  return (
    <div style={{ position: 'fixed', top: 64, left: 0, right: 0, zIndex: 1000 }}>
      <Alert
        message={
          <Space>
            <DatabaseOutlined />
            <Text strong>Demo Mode Active</Text>
            <Text type="secondary">({mockUsageCount} API{mockUsageCount !== 1 ? 's' : ''} using mock data)</Text>
          </Space>
        }
        description={
          <Text>
            The dashboard is using simulated data because the API server is unavailable or responding slowly.
            All functionality is demonstrated with mock data for testing purposes.
          </Text>
        }
        type="info"
        icon={<InfoCircleOutlined />}
        showIcon
        closable
        onClose={() => setShowBanner(false)}
        style={{ 
          borderRadius: 0,
          borderLeft: 'none',
          borderRight: 'none',
          borderTop: 'none'
        }}
      />
    </div>
  );
};

export default MockDataBanner;

