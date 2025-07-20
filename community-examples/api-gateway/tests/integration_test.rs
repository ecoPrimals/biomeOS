//! Integration tests for api-gateway primal

use api_gateway_primal::*;
use biomeos_primal_sdk::*;
use tokio;

#[tokio::test]
async fn test_primal_lifecycle() {
    let config = ApiGatewayConfig::default();
    let primal = ApiGateway::new(config);
    
    // Test initialization
    let sdk_config = PrimalConfig::default();
    assert!(primal.initialize(&sdk_config).await.is_ok());
    
    // Test health check
    let health = primal.health_check().await;
    assert_eq!(health.status, HealthStatus::Healthy);
    
    // Test request handling
    let request = PrimalRequest::new("ping", serde_json::json!({}));
    let response = primal.handle_request(request).await.unwrap();
    assert_eq!(response.status, ResponseStatus::Success);
    
    // Test shutdown
    assert!(primal.shutdown().await.is_ok());
}

#[tokio::test]
async fn test_invalid_request() {
    let config = ApiGatewayConfig::default();
    let primal = ApiGateway::new(config);
    
    let request = PrimalRequest::new("invalid_method", serde_json::json!({}));
    let result = primal.handle_request(request).await;
    
    assert!(result.is_err());
    match result.unwrap_err() {
        PrimalError::InvalidRequest(_) => {},
        _ => panic!("Expected InvalidRequest error"),
    }
}

#[tokio::test]
async fn test_metadata() {
    let config = ApiGatewayConfig::default();
    let primal = ApiGateway::new(config);
    
    let metadata = primal.metadata();
    assert_eq!(metadata.name, "api-gateway");
    assert_eq!(metadata.version, "0.1.0");
    assert!(!metadata.author.is_empty());
    assert!(!metadata.description.is_empty());
}

#[tokio::test]
async fn test_capabilities() {
    let config = ApiGatewayConfig::default();
    let primal = ApiGateway::new(config);
    
    let capabilities = primal.capabilities();
    assert!(!capabilities.is_empty());
    
    // Verify expected capabilities based on primal type
    // Add specific capability tests here
}
