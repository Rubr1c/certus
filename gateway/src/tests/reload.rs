use crate::config::parser;

#[tokio::test]
async fn reload_config_parses_valid_yaml() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("config.yaml");

    let yaml = r#"
server:
  port: 9090
routes:
  /api:
    endpoints:
      - "127.0.0.1:3000"
"#;
    std::fs::write(&path, yaml).unwrap();

    let config = parser::reload_config(path.to_str().unwrap()).await;

    assert!(config.is_ok());
    let config = config.unwrap();
    assert_eq!(config.server.port, 9090);
    assert!(config.routes.contains_key("/api"));
}

#[tokio::test]
async fn reload_config_rejects_invalid_yaml() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("config.yaml");

    std::fs::write(&path, "not: [valid: yaml: {{{}}}").unwrap();

    let config = parser::reload_config(path.to_str().unwrap()).await;

    assert!(config.is_err());
}

#[tokio::test]
async fn reload_config_rejects_missing_file() {
    let result =
        parser::reload_config("/tmp/nonexistent_certus_config_12345.yaml")
            .await;

    assert!(result.is_err());
}
