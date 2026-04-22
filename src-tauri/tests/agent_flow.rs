use std::fs;
use std::sync::Arc;
use tempfile::tempdir;
use agent_dock_lib::infrastructure::persistence::JsonAgentRepository;
use agent_dock_lib::services::agent_discovery_service::AgentDiscoveryService;
use agent_dock_lib::dto::agents::ScanTargetDto;

#[test]
fn test_agent_discovery_and_import_flow() {
    // 1. Setup temporary home directory
    let temp_home = tempdir().expect("create temp home");
    let home_path = temp_home.path();

    // Set environment variables to isolate from real user data
    std::env::set_var("USERPROFILE", home_path);
    std::env::set_var("HOME", home_path);

    // 2. Create a mock agent directory (e.g., .gemini)
    let gemini_root = home_path.join(".gemini");
    fs::create_dir_all(&gemini_root).expect("create .gemini dir");

    // 3. Define scan target
    let scan_targets = vec![ScanTargetDto {
        agent_type: "gemini".into(),
        name: "Gemini".into(),
        root_path: gemini_root.to_string_lossy().to_string(),
    }];

    // Setup service
    let agent_repo = Arc::new(JsonAgentRepository::new());
    let service = AgentDiscoveryService::new(agent_repo.clone());

    // 4. Scan for agents (Discovery)
    let candidates = service.scan_agents(scan_targets.clone());
    assert!(!candidates.is_empty(), "Should find at least one candidate");
    let candidate = &candidates[0];
    assert_eq!(candidate.agent_type, "gemini");
    assert_eq!(candidate.state, "ready");

    // 5. Import the agent
    let import_result = service.import_agents(
        vec![candidate.id.clone()],
        scan_targets.clone()
    ).expect("Import should succeed");

    assert_eq!(import_result.imported_agents.len(), 1);
    assert_eq!(import_result.imported_agents[0].agent_type, "gemini");
    assert!(import_result.imported_agents[0].managed);

    // 6. List managed agents
    let managed = service.list_managed_agents();
    assert!(managed.iter().any(|a| a.agent_type == Some("gemini".into())));

    // 7. List resolved agents and verify status
    let resolved = service.list_resolved_agents(scan_targets);
    let gemini_resolved = resolved.iter().find(|r| r.agent_type == "gemini").unwrap();
    assert!(gemini_resolved.managed);
    assert_eq!(gemini_resolved.status_label, "Managed");
}
