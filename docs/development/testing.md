# Testing Guidelines and Procedures

## Overview

This document provides comprehensive guidelines for testing the Wasmbed platform, including unit testing, integration testing, end-to-end testing, and performance testing.

## Testing Strategy

### Testing Pyramid

**Unit Tests (70%)**:
- Test individual functions and methods
- Mock external dependencies
- Fast execution
- High coverage

**Integration Tests (20%)**:
- Test component interactions
- Use real dependencies where possible
- Medium execution time
- Focus on critical paths

**End-to-End Tests (10%)**:
- Test complete workflows
- Use real system components
- Slower execution
- Validate user scenarios

### Testing Principles

**Test-Driven Development**:
- Write tests before implementation
- Red-Green-Refactor cycle
- Continuous testing
- Test coverage monitoring

**Quality Assurance**:
- Comprehensive test coverage
- Automated testing pipeline
- Performance testing
- Security testing

## Unit Testing

### Test Organization

**Test Structure**:
```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_function_success() {
        // Arrange
        let input = "test_input";
        let expected = "expected_output";
        
        // Act
        let result = function_under_test(input);
        
        // Assert
        assert_eq!(result, expected);
    }
    
    #[test]
    fn test_function_error() {
        // Arrange
        let input = "invalid_input";
        
        // Act & Assert
        assert!(function_under_test(input).is_err());
    }
}
```

**Test Naming Convention**:
- `test_function_name_success` - Test successful case
- `test_function_name_error` - Test error case
- `test_function_name_edge_case` - Test edge cases
- `test_function_name_invalid_input` - Test invalid input

### Mocking and Test Doubles

**Mock Implementation**:
```rust
use mockall::mock;

mock! {
    pub ExternalService {}
    
    impl ExternalServiceTrait for ExternalService {
        fn process_data(&self, data: &str) -> Result<String, Error>;
        fn get_status(&self) -> Status;
    }
}

#[test]
fn test_with_mock() {
    let mut mock_service = MockExternalService::new();
    mock_service.expect_process_data()
        .with(eq("test_data"))
        .times(1)
        .returning(|_| Ok("processed".to_string()));
    
    let result = function_using_service(&mock_service, "test_data");
    assert_eq!(result, "processed");
}
```

**Test Data Builders**:
```rust
pub struct TestDataBuilder {
    data: TestData,
}

impl TestDataBuilder {
    pub fn new() -> Self {
        Self {
            data: TestData::default(),
        }
    }
    
    pub fn with_field(mut self, value: String) -> Self {
        self.data.field = value;
        self
    }
    
    pub fn build(self) -> TestData {
        self.data
    }
}

#[test]
fn test_with_builder() {
    let test_data = TestDataBuilder::new()
        .with_field("test_value".to_string())
        .build();
    
    let result = function_under_test(&test_data);
    assert!(result.is_ok());
}
```

### Test Coverage

**Coverage Requirements**:
- Minimum 80% line coverage
- Minimum 90% branch coverage
- 100% coverage for critical functions
- All public APIs must be tested

**Coverage Tools**:
```bash
# Install coverage tools
cargo install cargo-tarpaulin

# Run coverage analysis
cargo tarpaulin --out Html

# Generate coverage report
cargo tarpaulin --out Html --output-dir coverage/
```

## Integration Testing

### Component Integration Tests

**Gateway Integration Tests**:
```rust
#[cfg(test)]
mod integration_tests {
    use super::*;
    use tokio::test;
    
    #[tokio::test]
    async fn test_gateway_device_enrollment() {
        // Setup
        let gateway = GatewayServer::new().await.unwrap();
        let device = create_test_device();
        
        // Test enrollment
        let result = gateway.enroll_device(&device).await;
        assert!(result.is_ok());
        
        // Verify device status
        let status = gateway.get_device_status(&device.id).await;
        assert_eq!(status, DeviceStatus::Enrolled);
    }
    
    #[tokio::test]
    async fn test_gateway_application_deployment() {
        // Setup
        let gateway = GatewayServer::new().await.unwrap();
        let application = create_test_application();
        let device = create_test_device();
        
        // Test deployment
        let result = gateway.deploy_application(&application, &device).await;
        assert!(result.is_ok());
        
        // Verify deployment status
        let status = gateway.get_application_status(&application.id).await;
        assert_eq!(status, ApplicationStatus::Running);
    }
}
```

**Controller Integration Tests**:
```rust
#[tokio::test]
async fn test_controller_reconciliation() {
    // Setup
    let controller = Controller::new().await.unwrap();
    let application = create_test_application();
    
    // Create application resource
    let result = controller.create_application(&application).await;
    assert!(result.is_ok());
    
    // Wait for reconciliation
    tokio::time::sleep(Duration::from_secs(1)).await;
    
    // Verify application status
    let status = controller.get_application_status(&application.id).await;
    assert_eq!(status, ApplicationStatus::Running);
}
```

### Database Integration Tests

**Kubernetes Integration Tests**:
```rust
#[tokio::test]
async fn test_kubernetes_crd_operations() {
    // Setup
    let client = Client::try_default().await.unwrap();
    let api: Api<Device> = Api::namespaced(client, "wasmbed");
    
    // Test CRD operations
    let device = create_test_device();
    let result = api.create(&PostParams::default(), &device).await;
    assert!(result.is_ok());
    
    // Test retrieval
    let retrieved = api.get(&device.metadata.name).await;
    assert!(retrieved.is_ok());
    
    // Test update
    let mut updated_device = retrieved.unwrap();
    updated_device.spec.device_type = "updated_type".to_string();
    let result = api.replace(&updated_device.metadata.name, &PostParams::default(), &updated_device).await;
    assert!(result.is_ok());
    
    // Test deletion
    let result = api.delete(&device.metadata.name, &DeleteParams::default()).await;
    assert!(result.is_ok());
}
```

## End-to-End Testing

### Complete Workflow Tests

**Device Enrollment E2E Test**:
```rust
#[tokio::test]
async fn test_device_enrollment_e2e() {
    // Setup
    let test_env = TestEnvironment::new().await.unwrap();
    
    // Start QEMU device
    let device = test_env.start_qemu_device().await.unwrap();
    
    // Test enrollment
    let result = test_env.enroll_device(&device).await;
    assert!(result.is_ok());
    
    // Verify device status in Kubernetes
    let status = test_env.get_device_status(&device.id).await;
    assert_eq!(status, DeviceStatus::Enrolled);
    
    // Test heartbeat
    let heartbeat_result = test_env.send_heartbeat(&device.id).await;
    assert!(heartbeat_result.is_ok());
    
    // Cleanup
    test_env.cleanup().await.unwrap();
}
```

**Application Deployment E2E Test**:
```rust
#[tokio::test]
async fn test_application_deployment_e2e() {
    // Setup
    let test_env = TestEnvironment::new().await.unwrap();
    let device = test_env.enroll_test_device().await.unwrap();
    
    // Create application
    let application = create_test_wasm_application();
    let result = test_env.create_application(&application).await;
    assert!(result.is_ok());
    
    // Wait for deployment
    test_env.wait_for_application_deployment(&application.id).await.unwrap();
    
    // Verify application running
    let status = test_env.get_application_status(&application.id).await;
    assert_eq!(status, ApplicationStatus::Running);
    
    // Test application execution
    let execution_result = test_env.execute_application_function(&application.id, "test_function").await;
    assert!(execution_result.is_ok());
    
    // Cleanup
    test_env.cleanup().await.unwrap();
}
```

### QEMU Integration Tests

**QEMU Device Tests**:
```rust
#[tokio::test]
async fn test_qemu_device_communication() {
    // Setup
    let qemu_manager = QemuManager::new().await.unwrap();
    
    // Start RISC-V device
    let riscv_device = qemu_manager.start_riscv_device().await.unwrap();
    
    // Test serial communication
    let message = "test_message";
    let result = riscv_device.send_message(message).await;
    assert!(result.is_ok());
    
    // Test response
    let response = riscv_device.receive_message().await;
    assert!(response.is_ok());
    
    // Stop device
    qemu_manager.stop_device(&riscv_device.id).await.unwrap();
}
```

**Multi-Device Tests**:
```rust
#[tokio::test]
async fn test_multi_device_coordination() {
    // Setup
    let test_env = TestEnvironment::new().await.unwrap();
    
    // Start multiple devices
    let devices = test_env.start_multiple_devices(3).await.unwrap();
    
    // Enroll all devices
    for device in &devices {
        let result = test_env.enroll_device(device).await;
        assert!(result.is_ok());
    }
    
    // Test device coordination
    let coordination_result = test_env.test_device_coordination(&devices).await;
    assert!(coordination_result.is_ok());
    
    // Cleanup
    test_env.cleanup().await.unwrap();
}
```

## Performance Testing

### Load Testing

**Gateway Load Tests**:
```rust
#[tokio::test]
async fn test_gateway_load() {
    // Setup
    let gateway = GatewayServer::new().await.unwrap();
    let clients = create_test_clients(100).await;
    
    // Test concurrent connections
    let tasks: Vec<_> = clients.into_iter()
        .map(|client| {
            tokio::spawn(async move {
                gateway.handle_client(client).await
            })
        })
        .collect();
    
    let results = futures::future::join_all(tasks).await;
    
    // Verify all connections successful
    for result in results {
        assert!(result.is_ok());
    }
}
```

**Application Performance Tests**:
```rust
#[tokio::test]
async fn test_application_performance() {
    // Setup
    let test_env = TestEnvironment::new().await.unwrap();
    let application = test_env.deploy_test_application().await.unwrap();
    
    // Test execution time
    let start = Instant::now();
    let result = test_env.execute_application_function(&application.id, "performance_test").await;
    let duration = start.elapsed();
    
    assert!(result.is_ok());
    assert!(duration < Duration::from_millis(100));
}
```

### Memory Testing

**Memory Leak Tests**:
```rust
#[tokio::test]
async fn test_memory_usage() {
    // Setup
    let gateway = GatewayServer::new().await.unwrap();
    let initial_memory = get_memory_usage();
    
    // Perform operations
    for i in 0..1000 {
        let device = create_test_device_with_id(i);
        let _ = gateway.enroll_device(&device).await;
    }
    
    // Check memory usage
    let final_memory = get_memory_usage();
    let memory_increase = final_memory - initial_memory;
    
    // Memory increase should be reasonable
    assert!(memory_increase < 100 * 1024 * 1024); // 100MB
}
```

## Security Testing

### Authentication Tests

**TLS Authentication Tests**:
```rust
#[tokio::test]
async fn test_tls_authentication() {
    // Setup
    let gateway = GatewayServer::new().await.unwrap();
    let valid_cert = create_valid_certificate();
    let invalid_cert = create_invalid_certificate();
    
    // Test valid authentication
    let result = gateway.authenticate_client(&valid_cert).await;
    assert!(result.is_ok());
    
    // Test invalid authentication
    let result = gateway.authenticate_client(&invalid_cert).await;
    assert!(result.is_err());
}
```

**Authorization Tests**:
```rust
#[tokio::test]
async fn test_authorization() {
    // Setup
    let gateway = GatewayServer::new().await.unwrap();
    let admin_user = create_admin_user();
    let regular_user = create_regular_user();
    
    // Test admin access
    let result = gateway.check_permission(&admin_user, "admin_action").await;
    assert!(result.is_ok());
    
    // Test regular user access
    let result = gateway.check_permission(&regular_user, "admin_action").await;
    assert!(result.is_err());
}
```

### Input Validation Tests

**Input Validation Tests**:
```rust
#[test]
fn test_input_validation() {
    // Test valid input
    let valid_input = "valid_input";
    let result = validate_input(valid_input);
    assert!(result.is_ok());
    
    // Test invalid input
    let invalid_inputs = vec![
        "", // Empty string
        &"a".repeat(1000), // Too long
        "invalid\x00chars", // Null bytes
        "invalid<script>", // Script tags
    ];
    
    for invalid_input in invalid_inputs {
        let result = validate_input(invalid_input);
        assert!(result.is_err());
    }
}
```

## Test Automation

### Continuous Integration

**GitHub Actions Workflow**:
```yaml
name: Tests

on:
  push:
    branches: [ main, develop ]
  pull_request:
    branches: [ main ]

jobs:
  test:
    runs-on: ubuntu-latest
    
    steps:
    - uses: actions/checkout@v3
    
    - name: Install Rust
      uses: actions-rs/toolchain@v1
      with:
        toolchain: stable
        components: rustfmt, clippy
    
    - name: Install QEMU
      run: |
        sudo apt-get update
        sudo apt-get install -y qemu-system-riscv32 qemu-system-arm qemu-system-xtensa
    
    - name: Run tests
      run: |
        cargo test
        cargo test --test integration
        cargo test --test e2e
    
    - name: Run clippy
      run: cargo clippy --all-targets --all-features -- -D warnings
    
    - name: Run fmt check
      run: cargo fmt --check
```

### Test Data Management

**Test Data Setup**:
```rust
pub struct TestDataManager {
    test_data: HashMap<String, TestData>,
}

impl TestDataManager {
    pub fn new() -> Self {
        Self {
            test_data: HashMap::new(),
        }
    }
    
    pub fn load_test_data(&mut self, name: &str, data: TestData) {
        self.test_data.insert(name.to_string(), data);
    }
    
    pub fn get_test_data(&self, name: &str) -> Option<&TestData> {
        self.test_data.get(name)
    }
    
    pub fn cleanup(&mut self) {
        self.test_data.clear();
    }
}
```

## Test Reporting

### Test Results

**Test Report Generation**:
```bash
# Run tests with report
cargo test -- --format=json | tee test-results.json

# Generate HTML report
cargo install cargo-test-report
cargo test-report --input test-results.json --output test-report.html
```

**Coverage Reports**:
```bash
# Generate coverage report
cargo tarpaulin --out Html --output-dir coverage/

# View coverage report
open coverage/tarpaulin-report.html
```

### Test Metrics

**Test Metrics Collection**:
- Test execution time
- Test coverage percentage
- Test pass/fail rates
- Performance benchmarks
- Memory usage statistics

**Test Quality Metrics**:
- Code coverage
- Branch coverage
- Function coverage
- Line coverage
- Test maintainability

## Best Practices

### Test Design

**Test Design Principles**:
- Write tests that are easy to understand
- Use descriptive test names
- Keep tests simple and focused
- Test one thing at a time
- Use appropriate assertions

**Test Maintenance**:
- Keep tests up to date
- Refactor tests when code changes
- Remove obsolete tests
- Maintain test documentation
- Regular test review

### Test Performance

**Performance Optimization**:
- Use parallel test execution
- Minimize test setup time
- Use efficient test data
- Avoid unnecessary I/O
- Cache test dependencies

**Test Reliability**:
- Make tests deterministic
- Avoid flaky tests
- Use proper synchronization
- Handle test cleanup
- Monitor test stability
