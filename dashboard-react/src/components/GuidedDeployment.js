import React, { useState } from 'react';
import {
  Modal,
  Steps,
  Form,
  Input,
  Select,
  Upload,
  Button,
  Card,
  Typography,
  Space,
  Alert,
  Divider,
  Row,
  Col,
  Tag,
  Progress,
} from 'antd';
import {
  CodeOutlined,
  UploadOutlined,
  PlayCircleOutlined,
  CheckCircleOutlined,
  InfoCircleOutlined,
  FileTextOutlined,
  SettingOutlined,
  RocketOutlined,
} from '@ant-design/icons';

const { Title, Paragraph, Text } = Typography;
const { Option } = Select;
const { TextArea } = Input;

const GuidedDeployment = ({ visible, onCancel, onSuccess }) => {
  const [currentStep, setCurrentStep] = useState(0);
  const [form] = Form.useForm();
  const [compilationStatus, setCompilationStatus] = useState('idle');
  const [compilationProgress, setCompilationProgress] = useState(0);
  const [compiledWasm, setCompiledWasm] = useState(null);
  const [sourceCode, setSourceCode] = useState('');

  const steps = [
    {
      title: 'Application Info',
      description: 'Basic application details',
      icon: <InfoCircleOutlined />,
    },
    {
      title: 'Source Code',
      description: 'Write or upload your code',
      icon: <CodeOutlined />,
    },
    {
      title: 'Compilation',
      description: 'Compile to WASM',
      icon: <SettingOutlined />,
    },
    {
      title: 'Deployment',
      description: 'Deploy to devices',
      icon: <RocketOutlined />,
    },
  ];

  const handleNext = () => {
    if (currentStep < steps.length - 1) {
      setCurrentStep(currentStep + 1);
    }
  };

  const handlePrev = () => {
    if (currentStep > 0) {
      setCurrentStep(currentStep - 1);
    }
  };

  const handleCompile = async () => {
    setCompilationStatus('compiling');
    setCompilationProgress(0);

    // Simulate compilation process
    const steps = [
      { progress: 20, message: 'Parsing source code...' },
      { progress: 40, message: 'Type checking...' },
      { progress: 60, message: 'Generating WASM bytecode...' },
      { progress: 80, message: 'Optimizing...' },
      { progress: 100, message: 'Compilation complete!' },
    ];

    for (const step of steps) {
      await new Promise(resolve => setTimeout(resolve, 1000));
      setCompilationProgress(step.progress);
      console.log(step.message);
    }

    setCompilationStatus('completed');
    setCompiledWasm('mock-wasm-bytecode-base64-encoded');
    console.log('WASM compilation successful!');
  };

  const handleDeploy = async () => {
    const values = await form.validateFields();
    console.log('Application deployed successfully!');
    onSuccess(values);
    onCancel();
  };

  const renderStepContent = () => {
    switch (currentStep) {
      case 0:
        return (
          <Card title="Application Information" style={{ marginBottom: 24 }}>
            <Form form={form} layout="vertical">
              <Form.Item
                name="name"
                label="Application Name"
                rules={[{ required: true, message: 'Please enter application name' }]}
              >
                <Input placeholder="e.g., sensor-monitor" />
              </Form.Item>
              
              <Form.Item
                name="description"
                label="Description"
                rules={[{ required: true, message: 'Please enter description' }]}
              >
                <TextArea 
                  rows={3} 
                  placeholder="Describe what this application does..."
                />
              </Form.Item>

              <Form.Item
                name="targetDevices"
                label="Target Devices"
                rules={[{ required: true, message: 'Please select target devices' }]}
              >
                <Select
                  mode="multiple"
                  placeholder="Select devices to deploy to"
                  style={{ width: '100%' }}
                >
                  <Option value="mcu-board-1">MCU Board 1 (riscv32)</Option>
                  <Option value="mcu-board-2">MCU Board 2 (riscv32)</Option>
                  <Option value="mcu-board-3">MCU Board 3 (riscv32)</Option>
                  <Option value="riscv-board-1">RISC-V Board 1 (riscv64)</Option>
                  <Option value="riscv-board-2">RISC-V Board 2 (riscv64)</Option>
                  <Option value="riscv-board-3">RISC-V Board 3 (riscv64)</Option>
                </Select>
              </Form.Item>
            </Form>
          </Card>
        );

      case 1:
        return (
          <Card title="Source Code" style={{ marginBottom: 24 }}>
            <Space direction="vertical" size="large" style={{ width: '100%' }}>
              <Alert
                message="Code Input Options"
                description="You can either write your code directly or upload a file. Supported languages: Rust, C/C++, AssemblyScript"
                type="info"
                showIcon
              />
              
              <div>
                <Title level={5}>Upload Source File</Title>
                <Upload.Dragger
                  name="file"
                  multiple={false}
                  accept=".rs,.c,.cpp,.ts,.wat,.wasm"
                  beforeUpload={() => false}
                  onChange={(info) => {
                    if (info.file) {
                      const reader = new FileReader();
                      reader.onload = (e) => {
                        setSourceCode(e.target.result);
                      };
                      reader.readAsText(info.file.originFileObj);
                    }
                  }}
                >
                  <p className="ant-upload-drag-icon">
                    <UploadOutlined />
                  </p>
                  <p className="ant-upload-text">Click or drag file to this area to upload</p>
                  <p className="ant-upload-hint">
                    Support for Rust (.rs), C/C++ (.c, .cpp), AssemblyScript (.ts), WAT (.wat), WASM (.wasm)
                  </p>
                </Upload.Dragger>
              </div>

              <Divider>OR</Divider>

              <div>
                <Title level={5}>Write Code Directly</Title>
                <TextArea
                  rows={15}
                  placeholder={`// Example Rust code for WASM
#[no_std]
#[no_main]

use core::panic::PanicInfo;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

#[no_mangle]
pub extern "C" fn add(a: i32, b: i32) -> i32 {
    a + b
}

#[no_mangle]
pub extern "C" fn main() {
    // Your application logic here
    let result = add(5, 3);
    // result = 8
}`}
                  value={sourceCode}
                  onChange={(e) => setSourceCode(e.target.value)}
                  style={{ fontFamily: 'monospace' }}
                />
              </div>
            </Space>
          </Card>
        );

      case 2:
        return (
          <Card title="WASM Compilation" style={{ marginBottom: 24 }}>
            <Space direction="vertical" size="large" style={{ width: '100%' }}>
              <Alert
                message="Compilation Process"
                description="Your source code will be compiled to WebAssembly bytecode for deployment to edge devices."
                type="info"
                showIcon
              />

              {compilationStatus === 'idle' && (
                <div style={{ textAlign: 'center', padding: '40px' }}>
                  <Button
                    type="primary"
                    size="large"
                    icon={<PlayCircleOutlined />}
                    onClick={handleCompile}
                    disabled={!sourceCode.trim()}
                  >
                    Start Compilation
                  </Button>
                  {!sourceCode.trim() && (
                    <div style={{ marginTop: 16, color: '#999' }}>
                      Please provide source code first
                    </div>
                  )}
                </div>
              )}

              {compilationStatus === 'compiling' && (
                <div style={{ textAlign: 'center', padding: '40px' }}>
                  <Progress
                    type="circle"
                    percent={compilationProgress}
                    status="active"
                    strokeColor={{
                      '0%': '#108ee9',
                      '100%': '#87d068',
                    }}
                  />
                  <div style={{ marginTop: 16 }}>
                    <Text>Compiling to WASM...</Text>
                  </div>
                </div>
              )}

              {compilationStatus === 'completed' && (
                <div>
                  <Alert
                    message="Compilation Successful!"
                    description="Your code has been successfully compiled to WASM bytecode."
                    type="success"
                    showIcon
                    style={{ marginBottom: 16 }}
                  />
                  
                  <Card size="small" title="Compiled WASM Info">
                    <Row gutter={16}>
                      <Col span={12}>
                        <Text strong>Size:</Text> <Tag color="green">2.4 KB</Tag>
                      </Col>
                      <Col span={12}>
                        <Text strong>Format:</Text> <Tag color="blue">WASM Binary</Tag>
                      </Col>
                    </Row>
                    <div style={{ marginTop: 8 }}>
                      <Text strong>Hash:</Text> <Text code>sha256:abc123...</Text>
                    </div>
                  </Card>
                </div>
              )}
            </Space>
          </Card>
        );

      case 3:
        return (
          <Card title="Deployment Configuration" style={{ marginBottom: 24 }}>
            <Space direction="vertical" size="large" style={{ width: '100%' }}>
              <Alert
                message="Ready to Deploy"
                description="Review your deployment configuration and deploy to selected devices."
                type="success"
                showIcon
              />

              <Card size="small" title="Deployment Summary">
                <Row gutter={16}>
                  <Col span={12}>
                    <Text strong>Application:</Text> <Text>{form.getFieldValue('name')}</Text>
                  </Col>
                  <Col span={12}>
                    <Text strong>WASM Size:</Text> <Tag color="green">2.4 KB</Tag>
                  </Col>
                </Row>
                <div style={{ marginTop: 8 }}>
                  <Text strong>Target Devices:</Text>
                  <div style={{ marginTop: 4 }}>
                    {form.getFieldValue('targetDevices')?.map(device => (
                      <Tag key={device} color="blue">{device}</Tag>
                    ))}
                  </div>
                </div>
              </Card>

              <div style={{ textAlign: 'center', padding: '20px' }}>
                <Button
                  type="primary"
                  size="large"
                  icon={<RocketOutlined />}
                  onClick={handleDeploy}
                  disabled={compilationStatus !== 'completed'}
                >
                  Deploy Application
                </Button>
              </div>
            </Space>
          </Card>
        );

      default:
        return null;
    }
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
      footer={[
        <Button key="cancel" onClick={onCancel}>
          Cancel
        </Button>,
        <Button key="prev" disabled={currentStep === 0} onClick={handlePrev}>
          Previous
        </Button>,
        <Button
          key="next"
          type="primary"
          disabled={currentStep === steps.length - 1}
          onClick={handleNext}
        >
          Next
        </Button>,
      ]}
    >
      <Steps current={currentStep} items={steps} style={{ marginBottom: 24 }} />
      {renderStepContent()}
    </Modal>
  );
};

export default GuidedDeployment;
