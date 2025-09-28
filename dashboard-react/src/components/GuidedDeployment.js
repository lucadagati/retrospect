import React, { useState } from 'react';
import {
  Modal,
  Steps,
  Form,
  Input,
  Select,
  Button,
  Space,
  Card,
  Typography,
  Row,
  Col,
  Divider,
  Alert,
  Upload,
  App,
  Tooltip,
  Tag,
} from 'antd';
import {
  CodeOutlined,
  RocketOutlined,
  CheckCircleOutlined,
  UploadOutlined,
  PlayCircleOutlined,
  QuestionCircleOutlined,
} from '@ant-design/icons';

const { Title, Paragraph, Text } = Typography;
const { Option } = Select;
const { TextArea } = Input;

const GuidedDeployment = ({ visible, onCancel, onSuccess }) => {
  const [currentStep, setCurrentStep] = useState(0);
  const [form] = Form.useForm();
  const [selectedTemplate, setSelectedTemplate] = useState(null);
  const [compilationStatus, setCompilationStatus] = useState('idle');
  const { message } = App.useApp();

  const testApplications = [
    {
      id: 'hello-world',
      name: 'Hello World',
      description: 'Simple greeting application that prints messages',
      language: 'Rust',
      code: `fn main() {
    println!("Hello from Wasmbed!");
    println!("Device is running successfully");
}`,
      features: ['Basic I/O', 'String handling', 'Console output'],
      size: '2KB',
      complexity: 'Beginner'
    },
    {
      id: 'led-blinker',
      name: 'LED Blinker',
      description: 'GPIO control example for blinking LEDs',
      language: 'Rust',
      code: `use std::thread;
use std::time::Duration;

fn main() {
    loop {
        // Turn LED on
        gpio_write(13, true);
        thread::sleep(Duration::from_millis(500));
        
        // Turn LED off
        gpio_write(13, false);
        thread::sleep(Duration::from_millis(500));
    }
}

fn gpio_write(pin: u8, value: bool) {
    // GPIO write implementation
}`,
      features: ['GPIO Control', 'Timing', 'Loop control'],
      size: '4KB',
      complexity: 'Intermediate'
    },
    {
      id: 'sensor-reader',
      name: 'Sensor Reader',
      description: 'ADC sensor reading and data processing',
      language: 'Rust',
      code: `use std::time::Duration;
use std::thread;

fn main() {
    loop {
        let temperature = read_adc(0);
        let humidity = read_adc(1);
        
        println!("Temperature: {}°C", temperature);
        println!("Humidity: {}%", humidity);
        
        thread::sleep(Duration::from_secs(1));
    }
}

fn read_adc(channel: u8) -> f32 {
    // ADC read implementation
    25.5 // Mock value
}`,
      features: ['ADC Reading', 'Data Processing', 'Sensor Integration'],
      size: '6KB',
      complexity: 'Intermediate'
    },
    {
      id: 'network-test',
      name: 'Network Test',
      description: 'Network connectivity and communication test',
      language: 'Rust',
      code: `use std::net::TcpStream;
use std::io::Write;

fn main() {
    match TcpStream::connect("gateway:30430") {
        Ok(mut stream) => {
            let message = "Hello Gateway!";
            stream.write_all(message.as_bytes()).unwrap();
            println!("Connected to gateway successfully");
        }
        Err(e) => {
            println!("Failed to connect: {}", e);
        }
    }
}`,
      features: ['Network Communication', 'TCP/IP', 'Error Handling'],
      size: '8KB',
      complexity: 'Advanced'
    }
  ];

  const steps = [
    {
      title: 'Select Template',
      description: 'Choose a test application template',
      icon: <CodeOutlined />
    },
    {
      title: 'Configure',
      description: 'Set application parameters',
      icon: <RocketOutlined />
    },
    {
      title: 'Compile',
      description: 'Build WASM bytecode',
      icon: <CheckCircleOutlined />
    },
    {
      title: 'Deploy',
      description: 'Deploy to target devices',
      icon: <PlayCircleOutlined />
    }
  ];

  const handleTemplateSelect = (templateId) => {
    const template = testApplications.find(t => t.id === templateId);
    setSelectedTemplate(template);
    form.setFieldsValue({
      name: template.name.toLowerCase().replace(/\s+/g, '-'),
      description: template.description,
      language: template.language
    });
  };

  const handleCompile = async () => {
    setCompilationStatus('compiling');
    
    // Simulate compilation process
    setTimeout(() => {
      setCompilationStatus('success');
      message.success('WASM compilation completed successfully!');
    }, 3000);
  };

  const handleDeploy = async (values) => {
    try {
      // Create application with compiled WASM
      const application = {
        name: values.name,
        description: values.description,
        language: values.language,
        template: selectedTemplate.id,
        wasmBytes: 'compiled-wasm-bytes', // In real implementation, this would be the actual WASM
        targetDevices: values.targetDevices || ['all_devices']
      };

      // Simulate deployment
      message.success(`Application "${application.name}" deployed successfully!`);
      onSuccess(application);
      onCancel();
    } catch (error) {
      message.error('Deployment failed');
    }
  };

  const renderStepContent = () => {
    switch (currentStep) {
      case 0:
        return (
          <div>
            <Title level={4}>Select Test Application Template</Title>
            <Paragraph>
              Choose from pre-built test applications to get started quickly. Each template includes source code and is ready to compile.
            </Paragraph>
            
            <Row gutter={[16, 16]}>
              {testApplications.map(template => (
                <Col xs={24} sm={12} lg={6} key={template.id}>
                  <Card
                    hoverable
                    style={{
                      border: selectedTemplate?.id === template.id ? '2px solid #1890ff' : '1px solid #d9d9d9',
                      background: selectedTemplate?.id === template.id ? '#f0f9ff' : '#ffffff'
                    }}
                    onClick={() => handleTemplateSelect(template.id)}
                  >
                    <div style={{ textAlign: 'center' }}>
                      <CodeOutlined style={{ fontSize: '24px', color: '#1890ff', marginBottom: '8px' }} />
                      <Title level={5} style={{ margin: '8px 0' }}>{template.name}</Title>
                      <Paragraph style={{ fontSize: '12px', margin: '8px 0' }}>
                        {template.description}
                      </Paragraph>
                      <div style={{ marginTop: '12px' }}>
                        <Tag color="blue">{template.language}</Tag>
                        <Tag color="green">{template.complexity}</Tag>
                        <Tag color="orange">{template.size}</Tag>
                      </div>
                      <div style={{ marginTop: '8px' }}>
                        {template.features.map(feature => (
                          <Tag key={feature} size="small" color="default">{feature}</Tag>
                        ))}
                      </div>
                    </div>
                  </Card>
                </Col>
              ))}
            </Row>
          </div>
        );

      case 1:
        return (
          <div>
            <Title level={4}>Configure Application</Title>
            <Paragraph>
              Customize your application settings and target devices.
            </Paragraph>
            
            <Form
              form={form}
              layout="vertical"
              initialValues={{
                name: selectedTemplate?.name.toLowerCase().replace(/\s+/g, '-'),
                description: selectedTemplate?.description,
                language: selectedTemplate?.language
              }}
            >
              <Row gutter={16}>
                <Col span={12}>
                  <Form.Item
                    name="name"
                    label={
                      <Space>
                        <span>Application Name</span>
                        <Tooltip title="Unique identifier for your application">
                          <QuestionCircleOutlined style={{ color: '#1890ff' }} />
                        </Tooltip>
                      </Space>
                    }
                    rules={[{ required: true, message: 'Please enter application name' }]}
                  >
                    <Input placeholder="Enter application name" />
                  </Form.Item>
                </Col>
                <Col span={12}>
                  <Form.Item
                    name="language"
                    label="Programming Language"
                  >
                    <Input disabled />
                  </Form.Item>
                </Col>
              </Row>
              
              <Form.Item
                name="description"
                label="Description"
              >
                <TextArea rows={3} placeholder="Enter application description" />
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
                  <Option value="esp32-devices">ESP32 Devices</Option>
                  <Option value="stm32-devices">STM32 Devices</Option>
                  <Option value="riscv-devices">RISC-V Devices</Option>
                </Select>
              </Form.Item>
            </Form>
          </div>
        );

      case 2:
        return (
          <div>
            <Title level={4}>Compile to WASM</Title>
            <Paragraph>
              Compile your source code to WebAssembly bytecode for deployment.
            </Paragraph>
            
            {selectedTemplate && (
              <Card title="Source Code" size="small" style={{ marginBottom: '16px' }}>
                <pre style={{ 
                  background: '#f5f5f5', 
                  padding: '12px', 
                  borderRadius: '4px',
                  fontSize: '12px',
                  overflow: 'auto',
                  maxHeight: '200px'
                }}>
                  {selectedTemplate.code}
                </pre>
              </Card>
            )}
            
            <div style={{ textAlign: 'center', padding: '40px 0' }}>
              {compilationStatus === 'idle' && (
                <Button
                  type="primary"
                  size="large"
                  icon={<RocketOutlined />}
                  onClick={handleCompile}
                >
                  Start Compilation
                </Button>
              )}
              
              {compilationStatus === 'compiling' && (
                <div>
                  <div style={{ fontSize: '16px', marginBottom: '16px' }}>
                    Compiling to WASM...
                  </div>
                  <div style={{ color: '#1890ff' }}>
                    <RocketOutlined spin style={{ fontSize: '24px' }} />
                  </div>
                </div>
              )}
              
              {compilationStatus === 'success' && (
                <div>
                  <CheckCircleOutlined style={{ fontSize: '48px', color: '#52c41a', marginBottom: '16px' }} />
                  <div style={{ fontSize: '16px', color: '#52c41a' }}>
                    Compilation Successful!
                  </div>
                  <div style={{ fontSize: '12px', color: '#666', marginTop: '8px' }}>
                    WASM bytecode ready for deployment
                  </div>
                </div>
              )}
            </div>
          </div>
        );

      case 3:
        return (
          <div>
            <Title level={4}>Deploy Application</Title>
            <Paragraph>
              Deploy your compiled WASM application to the selected target devices.
            </Paragraph>
            
            <Alert
              message="Ready to Deploy"
              description={`Application "${form.getFieldValue('name')}" is ready to be deployed to the selected devices.`}
              type="success"
              showIcon
              style={{ marginBottom: '24px' }}
            />
            
            <Card title="Deployment Summary" size="small">
              <Row gutter={16}>
                <Col span={8}>
                  <div style={{ textAlign: 'center' }}>
                    <div style={{ fontSize: '24px', fontWeight: 'bold', color: '#1890ff' }}>
                      {selectedTemplate?.name}
                    </div>
                    <Text type="secondary">Application</Text>
                  </div>
                </Col>
                <Col span={8}>
                  <div style={{ textAlign: 'center' }}>
                    <div style={{ fontSize: '24px', fontWeight: 'bold', color: '#52c41a' }}>
                      {selectedTemplate?.size}
                    </div>
                    <Text type="secondary">WASM Size</Text>
                  </div>
                </Col>
                <Col span={8}>
                  <div style={{ textAlign: 'center' }}>
                    <div style={{ fontSize: '24px', fontWeight: 'bold', color: '#722ed1' }}>
                      {form.getFieldValue('targetDevices')?.length || 0}
                    </div>
                    <Text type="secondary">Target Devices</Text>
                  </div>
                </Col>
              </Row>
            </Card>
          </div>
        );

      default:
        return null;
    }
  };

  const handleNext = () => {
    if (currentStep === 0 && !selectedTemplate) {
      message.warning('Please select a template first');
      return;
    }
    if (currentStep === 2 && compilationStatus !== 'success') {
      message.warning('Please complete compilation first');
      return;
    }
    setCurrentStep(currentStep + 1);
  };

  const handlePrev = () => {
    setCurrentStep(currentStep - 1);
  };

  const handleFinish = () => {
    form.validateFields().then(values => {
      handleDeploy(values);
    }).catch(error => {
      console.error('Validation failed:', error);
    });
  };

  return (
    <Modal
      title={
        <Space>
          <RocketOutlined style={{ color: '#1890ff' }} />
          <span>Guided Application Deployment</span>
        </Space>
      }
      open={visible}
      onCancel={onCancel}
      width={800}
      footer={null}
    >
      <Steps current={currentStep} items={steps} style={{ marginBottom: '24px' }} />
      
      <div style={{ minHeight: '400px' }}>
        {renderStepContent()}
      </div>
      
      <Divider />
      
      <div style={{ textAlign: 'right' }}>
        <Space>
          {currentStep > 0 && (
            <Button onClick={handlePrev}>
              Previous
            </Button>
          )}
          
          {currentStep < steps.length - 1 ? (
            <Button type="primary" onClick={handleNext}>
              Next
            </Button>
          ) : (
            <Button 
              type="primary" 
              onClick={handleFinish}
              disabled={compilationStatus !== 'success'}
            >
              Deploy Application
            </Button>
          )}
          
          <Button onClick={onCancel}>
            Cancel
          </Button>
        </Space>
      </div>
    </Modal>
  );
};

export default GuidedDeployment;