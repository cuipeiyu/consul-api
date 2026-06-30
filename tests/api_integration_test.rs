use consul_api::*;
use std::time::Duration;

// Helper function to create test client
fn create_test_client() -> Client {
    Client::new()
}

// ==================== Client Tests ====================

#[tokio::test]
async fn test_client_creation() {
    let _client = Client::new();
    // Just test that client can be created
    assert!(true);
}

// ==================== Status API Tests (最简单的 API) ====================

#[tokio::test]
async fn test_status_leader() {
    let client = create_test_client();
    
    let result = client.status_leader(&Default::default()).await;
    assert!(result.is_ok(), "status_leader failed: {:?}", result.err());
    let leader = result.unwrap();
    assert!(!leader.is_empty(), "Leader should not be empty");
    println!("Leader: {}", leader);
}

#[tokio::test]
async fn test_status_peers() {
    let client = create_test_client();
    
    let result = client.status_peers(&Default::default()).await;
    assert!(result.is_ok(), "status_peers failed: {:?}", result.err());
    let peers = result.unwrap();
    assert!(!peers.is_empty(), "Peers should not be empty");
    println!("Peers count: {}", peers.len());
}

// ==================== Agent API Tests ====================

#[tokio::test]
async fn test_agent_checks() {
    let client = create_test_client();
    
    let result = client.agent_checks(&Default::default()).await;
    assert!(result.is_ok(), "agent_checks failed: {:?}", result.err());
    println!("Agent checks count: {}", result.unwrap().len());
}

#[tokio::test]
async fn test_agent_services() {
    let client = create_test_client();
    
    let result = client.agent_services(&Default::default()).await;
    assert!(result.is_ok(), "agent_services failed: {:?}", result.err());
    println!("Agent services count: {}", result.unwrap().len());
}

#[tokio::test]
async fn test_agent_check_register_and_deregister() {
    let client = create_test_client();
    let check_id = "test-check-12345";
    
    // Register a check
    let check_def = CheckDefinition {
        id: check_id.to_string(),
        name: "Test Check".to_string(),
        interval: Some("10s".to_string()),
        http: "http://127.0.0.1:8500".to_string(),
        ..Default::default()
    };
    
    let result = client.agent_check_register(&check_def).await;
    assert!(result.is_ok(), "agent_check_register failed: {:?}", result.err());
    assert_eq!(result.unwrap(), true, "Should return true on success");
    
    // Wait a bit
    tokio::time::sleep(Duration::from_secs(1)).await;
    
    // Deregister the check
    let query = DeregisterCheckRequestQuery {
        check_id: check_id.to_string(),
    };
    let result = client.agent_check_deregister(&query).await;
    assert!(result.is_ok(), "agent_check_deregister failed: {:?}", result.err());
    assert_eq!(result.unwrap(), true, "Should return true on success");
}

#[tokio::test]
async fn test_agent_check_ttl_operations() {
    let client = create_test_client();
    let check_id = "test-check-ttl-12345";
    
    // First register a TTL check
    let check_def = CheckDefinition {
        id: check_id.to_string(),
        name: "Test TTL Check".to_string(),
        ttl: Some("60s".to_string()),
        ..Default::default()
    };
    
    let result = client.agent_check_register(&check_def).await;
    assert!(result.is_ok(), "Failed to register TTL check: {:?}", result.err());
    
    // Test pass
    let query = AgentTTLCheckRequestQuery {
        check_id: check_id.to_string(),
        note: None,
    };
    let result = client.agent_check_pass(&query).await;
    assert!(result.is_ok(), "agent_check_pass failed: {:?}", result.err());
    assert_eq!(result.unwrap(), true, "Should return true on success");
    
    // Test warn
    let result = client.agent_check_warn(&query).await;
    assert!(result.is_ok(), "agent_check_warn failed: {:?}", result.err());
    assert_eq!(result.unwrap(), true, "Should return true on success");
    
    // Test fail
    let result = client.agent_check_fail(&query).await;
    assert!(result.is_ok(), "agent_check_fail failed: {:?}", result.err());
    assert_eq!(result.unwrap(), true, "Should return true on success");
    
    // Deregister
    let dereg_query = DeregisterCheckRequestQuery {
        check_id: check_id.to_string(),
    };
    let _ = client.agent_check_deregister(&dereg_query).await;
}

// ==================== Catalog API Tests ====================

#[tokio::test]
async fn test_catalog_list_datacenters() {
    let client = create_test_client();
    
    let result = client.catalog_list_datacenters().await;
    assert!(result.is_ok(), "catalog_list_datacenters failed: {:?}", result.err());
    let datacenters = result.unwrap();
    assert!(!datacenters.is_empty(), "Should have at least one datacenter");
    println!("Datacenters: {:?}", datacenters);
}

#[tokio::test]
async fn test_catalog_list_nodes() {
    let client = create_test_client();
    
    let result = client.catalog_list_nodes().await;
    assert!(result.is_ok(), "catalog_list_nodes failed: {:?}", result.err());
    let nodes = result.unwrap();
    println!("Catalog nodes count: {}", nodes.len());
}

#[tokio::test]
async fn test_catalog_list_services() {
    let client = create_test_client();
    
    let result = client.catalog_list_services(&Default::default()).await;
    assert!(result.is_ok(), "catalog_list_services failed: {:?}", result.err());
    let services = result.unwrap();
    println!("Catalog services count: {}", services.len());
}

#[tokio::test]
async fn test_catalog_list_nodes_for_service() {
    let client = create_test_client();
    
    // Use "consul" service which should always exist
    let result = client.catalog_list_nodes_for_service("consul", &Default::default()).await;
    assert!(result.is_ok(), "catalog_list_nodes_for_service failed: {:?}", result.err());
    let nodes = result.unwrap();
    println!("Nodes for service 'consul' count: {}", nodes.len());
}

// ==================== Health API Tests ====================

#[tokio::test]
async fn test_health_list_nodes() {
    let client = create_test_client();
    
    // Use "consul" service which should always exist
    let result = client.health_list_nodes("consul", &Default::default()).await;
    assert!(result.is_ok(), "health_list_nodes failed: {:?}", result.err());
    let nodes = result.unwrap();
    println!("Health nodes for 'consul' count: {}", nodes.len());
}

#[tokio::test]
async fn test_health_list_services() {
    let client = create_test_client();
    
    let result = client.health_list_services("consul", &Default::default()).await;
    assert!(result.is_ok(), "health_list_services failed: {:?}", result.err());
    let services = result.unwrap();
    println!("Health services count: {}", services.len());
}

#[tokio::test]
async fn test_health_list_state() {
    let client = create_test_client();
    
    let result = client.health_list_state(Health::Passing, &Default::default()).await;
    assert!(result.is_ok(), "health_list_state failed: {:?}", result.err());
    let states = result.unwrap();
    println!("Health state 'passing' count: {}", states.len());
}

// ==================== KV API Tests ====================

#[tokio::test]
async fn test_kv_operations() {
    let client = create_test_client();
    let test_key = "test-integration-key";
    let test_value = b"test-value";
    
    // Test create/update
    let result = client.kv_create_or_update_key(test_key, test_value.to_vec(), &Default::default()).await;
    assert!(result.is_ok(), "kv_create_or_update_key failed: {:?}", result.err());
    assert_eq!(result.unwrap(), true, "Should return true on success");
    
    // Test read with raw=true to get raw value
    let read_query = KVReadKeyQuery {
        raw: Some(true),
        ..Default::default()
    };
    let result = client.kv_read_key(test_key, &read_query).await;
    assert!(result.is_ok(), "kv_read_key failed: {:?}", result.err());
    
    if let Ok(Some(value)) = result {
        assert_eq!(&value, &test_value, "KV value mismatch");
        println!("KV read successful, value: {:?}", String::from_utf8_lossy(&value));
    } else {
        panic!("KV value should exist");
    }
    
    // Test delete
    let result = client.kv_delete_key(test_key, &Default::default()).await;
    assert!(result.is_ok(), "kv_delete_key failed: {:?}", result.err());
    assert_eq!(result.unwrap(), true, "Should return true on success");
}

#[tokio::test]
async fn test_kv_read_nonexistent_key() {
    let client = create_test_client();
    
    let result = client.kv_read_key("non-existent-key-12345", &Default::default()).await;
    assert!(result.is_ok(), "kv_read_key should not error for non-existent key");
    assert!(result.unwrap().is_none(), "Should return None for non-existent key");
}

// ==================== Event API Tests ====================

#[tokio::test]
async fn test_event_fire_and_list() {
    let client = create_test_client();
    let event_name = "test-event-12345";
    
    // Fire an event
    let result = client.event_fire(event_name, None, &Default::default()).await;
    assert!(result.is_ok(), "event_fire failed: {:?}", result.err());
    println!("Event fire result: {:?}", result.unwrap());
    
    // Wait a bit for event to propagate
    tokio::time::sleep(Duration::from_secs(2)).await;
    
    // List events
    let result = client.event_list(&Default::default()).await;
    assert!(result.is_ok(), "event_list failed: {:?}", result.err());
    
    let events = result.unwrap();
    assert!(!events.is_empty(), "Should have at least one event");
    println!("Events count: {}", events.len());
}

// ==================== Agent Service Tests ====================

#[tokio::test]
async fn test_agent_service_register_and_deregister() {
    let client = create_test_client();
    let service_id = "test-service-12345";
    
    // Register a service without check
    let service_def = ServiceDefinition {
        id: service_id.to_string(),
        name: "test-service".to_string(),
        tags: vec!["test".to_string()],
        address: "127.0.0.1".to_string(),
        port: 8080,
        // Don't set check field to avoid TTL error
        ..Default::default()
    };
    
    let query = RegisterServiceRequestQuery::default();
    
    let result = client.agent_register_service(&query, &service_def).await;
    assert!(result.is_ok(), "agent_register_service failed: {:?}", result.err());
    assert_eq!(result.unwrap(), true, "Should return true on success");
    
    // Deregister the service
    let dereg_query = DeregisterServiceRequestQuery::default();
    let result = client.agent_deregister_service(service_id, &dereg_query).await;
    assert!(result.is_ok(), "agent_deregister_service failed: {:?}", result.err());
    assert_eq!(result.unwrap(), true, "Should return true on success");
}

#[tokio::test]
async fn test_agent_service_configuration() {
    let client = create_test_client();
    
    // This will likely return None if service doesn't exist
    let query = ServiceConfigurationRequestQuery {
        service_id: "non-existent-service".to_string(),
    };
    let result = client.agent_service_configuration(&query).await;
    assert!(result.is_ok(), "agent_service_configuration failed: {:?}", result.err());
    
    match result.unwrap() {
        Some(service) => println!("Found service: {:?}", service),
        None => println!("Service not found (expected for non-existent service)"),
    }
}

#[tokio::test]
async fn test_agent_get_service_health_by_name() {
    let client = create_test_client();
    
    let result = client.agent_get_service_health_by_name("consul", &Default::default()).await;
    assert!(result.is_ok(), "agent_get_service_health_by_name failed: {:?}", result.err());
    println!("Service health by name 'consul': {:?}", result.unwrap());
}

// ==================== Additional Health API Tests ====================

#[tokio::test]
async fn test_health_list_service_instances() {
    let client = create_test_client();
    
    let result = client.health_list_service_instances("consul", &Default::default()).await;
    assert!(result.is_ok(), "health_list_service_instances failed: {:?}", result.err());
    let instances = result.unwrap();
    println!("Service instances for 'consul' count: {}", instances.len());
}

// ==================== Additional Catalog API Tests ====================

#[tokio::test]
async fn test_catalog_node_services() {
    let client = create_test_client();
    
    // Get the first node from catalog
    let nodes_result = client.catalog_list_nodes().await;
    assert!(nodes_result.is_ok(), "Failed to list nodes");
    
    if let Ok(nodes) = nodes_result {
        if let Some(first_node) = nodes.first() {
            let result = client.catalog_node_services(&first_node.node, &Default::default()).await;
            assert!(result.is_ok(), "catalog_node_services failed: {:?}", result.err());
            println!("Services for node {:?}: {:?}", first_node.node, result.unwrap());
        }
    }
}

// ==================== Additional Agent API Tests ====================

#[tokio::test]
async fn test_agent_check_update() {
    let client = create_test_client();
    let check_id = "test-check-update-12345";
    
    // First register a TTL check
    let check_def = CheckDefinition {
        id: check_id.to_string(),
        name: "Test TTL Check for Update".to_string(),
        ttl: Some("60s".to_string()),
        ..Default::default()
    };
    
    let result = client.agent_check_register(&check_def).await;
    assert!(result.is_ok(), "Failed to register TTL check: {:?}", result.err());
    
    // Update the check
    let query = AgentTTLCheckUpdateRequestQuery {
        check_id: check_id.to_string(),
    };
    let body = AgentTTLCheckUpdateRequestBody {
        status: Some("passing".to_string()),
        output: Some("Test output".to_string()),
    };
    
    let result = client.agent_check_update(&query, &body).await;
    assert!(result.is_ok(), "agent_check_update failed: {:?}", result.err());
    assert_eq!(result.unwrap(), true, "Should return true on success");
    
    // Clean up
    let dereg_query = DeregisterCheckRequestQuery {
        check_id: check_id.to_string(),
    };
    let _ = client.agent_check_deregister(&dereg_query).await;
}

#[tokio::test]
async fn test_agent_enable_maintenance_mode() {
    let client = create_test_client();
    let service_id = "test-service-maintenance-12345";
    
    // First register a service
    let service_def = ServiceDefinition {
        id: service_id.to_string(),
        name: "test-service".to_string(),
        address: "127.0.0.1".to_string(),
        port: 8080,
        ..Default::default()
    };
    
    let query = RegisterServiceRequestQuery::default();
    let result = client.agent_register_service(&query, &service_def).await;
    assert!(result.is_ok(), "Failed to register service: {:?}", result.err());
    
    // Enable maintenance mode
    let maint_query = EnableMaintenanceModeRequestQuery {
        service_id: service_id.to_string(),
        enable: true,
        reason: Some("Testing maintenance mode".to_string()),
    };
    
    let result = client.agent_enable_maintenance_mode(&maint_query).await;
    assert!(result.is_ok(), "agent_enable_maintenance_mode (enable) failed: {:?}", result.err());
    assert_eq!(result.unwrap(), true, "Should return true on success");
    
    // Wait a bit
    tokio::time::sleep(Duration::from_secs(1)).await;
    
    // Disable maintenance mode
    let maint_query = EnableMaintenanceModeRequestQuery {
        service_id: service_id.to_string(),
        enable: false,
        reason: Some("Done testing".to_string()),
    };
    
    let result = client.agent_enable_maintenance_mode(&maint_query).await;
    assert!(result.is_ok(), "agent_enable_maintenance_mode (disable) failed: {:?}", result.err());
    assert_eq!(result.unwrap(), true, "Should return true on success");
    
    // Clean up
    let dereg_query = DeregisterServiceRequestQuery::default();
    let _ = client.agent_deregister_service(service_id, &dereg_query).await;
}

#[tokio::test]
async fn test_agent_connect_authorize() {
    let client = create_test_client();
    
    // This will likely fail because Connect might not be enabled
    // But we test that the API call doesn't panic
    let query = ConnectAuthorizeRequestQuery::default();
    let body = ConnectAuthorizeRequest {
        target: "test".to_string(),
        client_cert_uri: "".to_string(),
        client_cert_serial: "".to_string(),
    };
    
    let result = client.agent_connect_authorize(&query, &body).await;
    // This might fail if Connect is not enabled, but should not panic
    println!("agent_connect_authorize result: {:?}", result);
    // We don't assert here because Connect might not be enabled
}

// ==================== Catalog Register/Deregister Entity Tests ====================

#[tokio::test]
async fn test_catalog_register_and_deregister_entity() {
    let client = create_test_client();
    let node_name = "test-node-12345";
    
    // Register a node
    let register_req = RegisterRequest {
        node: node_name.to_string(),
        address: "192.168.1.100".to_string(),
        ..Default::default()
    };
    
    let query = CatalogRegisterEntityQuery::default();
    let result = client.catalog_register_entity(&query, &register_req).await;
    assert!(result.is_ok(), "catalog_register_entity failed: {:?}", result.err());
    assert_eq!(result.unwrap(), true, "Should return true on success");
    
    // Wait a bit
    tokio::time::sleep(Duration::from_secs(1)).await;
    
    // Deregister the node
    let deregister_req = DeregisterRequest {
        node: node_name.to_string(),
        ..Default::default()
    };
    
    let query = CatalogDeregisterEntityQuery::default();
    let result = client.catalog_deregister_entity(&query, &deregister_req).await;
    assert!(result.is_ok(), "catalog_deregister_entity failed: {:?}", result.err());
    assert_eq!(result.unwrap(), true, "Should return true on success");
}

// ==================== Mesh and Gateway Tests ====================

#[tokio::test]
async fn test_catalog_list_nodes_for_mesh_capable_service() {
    let client = create_test_client();
    
    // Try with "consul" service (may or may not be mesh-capable)
    let result = client.catalog_list_nodes_for_mesh_capable_service("consul", &Default::default()).await;
    assert!(result.is_ok(), "catalog_list_nodes_for_mesh_capable_service failed: {:?}", result.err());
    let nodes = result.unwrap();
    println!("Mesh-capable nodes for 'consul' count: {}", nodes.len());
}

#[tokio::test]
async fn test_catalog_gateway_services() {
    let client = create_test_client();
    
    // This might return empty if no gateways are configured
    let result = client.catalog_gateway_services("consul", &Default::default()).await;
    assert!(result.is_ok(), "catalog_gateway_services failed: {:?}", result.err());
    let services = result.unwrap();
    println!("Gateway services count: {}", services.len());
}

#[tokio::test]
async fn test_health_list_service_instances_for_mesh_capable() {
    let client = create_test_client();
    
    // Try with "consul" service
    let result = client.health_list_service_instances_for_mesh_capable("consul", &Default::default()).await;
    assert!(result.is_ok(), "health_list_service_instances_for_mesh_capable failed: {:?}", result.err());
    let instances = result.unwrap();
    println!("Mesh-capable service instances for 'consul' count: {}", instances.len());
}

#[tokio::test]
async fn test_health_list_service_instances_for_ingress_gateways() {
    let client = create_test_client();
    
    // This might return empty if no ingress gateways are configured
    let result = client.health_list_service_instances_for_ingress_gateways("consul", &Default::default()).await;
    assert!(result.is_ok(), "health_list_service_instances_for_ingress_gateways failed: {:?}", result.err());
    let instances = result.unwrap();
    println!("Ingress gateway service instances for 'consul' count: {}", instances.len());
}

// ==================== Additional Health API Tests ====================

#[tokio::test]
async fn test_agent_get_service_health_by_id() {
    let client = create_test_client();
    
    // First register a service to test with
    let service_id = "test-service-health-id-12345";
    let service_def = ServiceDefinition {
        id: service_id.to_string(),
        name: "test-service".to_string(),
        address: "127.0.0.1".to_string(),
        port: 8080,
        ..Default::default()
    };
    
    let query = RegisterServiceRequestQuery::default();
    let result = client.agent_register_service(&query, &service_def).await;
    assert!(result.is_ok(), "Failed to register service: {:?}", result.err());
    
    // Wait a bit for service to be registered
    tokio::time::sleep(Duration::from_secs(2)).await;
    
    // Get service health by ID
    let result = client.agent_get_service_health_by_id(service_id, &Default::default()).await;
    assert!(result.is_ok(), "agent_get_service_health_by_id failed: {:?}", result.err());
    
    match result.unwrap() {
        Some(health) => println!("Service health by ID {:?}: {:?}", service_id, health),
        None => println!("Service health not found (might not have checks)"),
    }
    
    // Clean up
    let dereg_query = DeregisterServiceRequestQuery::default();
    let _ = client.agent_deregister_service(service_id, &dereg_query).await;
}
