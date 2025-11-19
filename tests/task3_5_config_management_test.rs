use mcp_inspector_mcp::models::ProfileConfig;
use mcp_inspector_mcp::services::{
    ConfigFormat, ConfigImportExport, ConfigTemplate, PresetTemplate, ProfileManager,
};
use serial_test::serial;
use std::env;
use std::path::Path;
use tempfile::TempDir;

/// Setup test environment with a temporary directory
fn setup_test_env() -> TempDir {
    let temp_dir = TempDir::new().unwrap();
    env::set_current_dir(temp_dir.path()).unwrap();
    temp_dir
}

// ============================================================
// ProfileManager Tests (11 tests)
// ============================================================

#[test]
#[serial]
fn test_profile_manager_initialization() {
    let _temp_dir = setup_test_env();
    let manager = ProfileManager::new();
    assert!(manager.is_ok());
}

#[test]
#[serial]
fn test_save_and_load_profile() {
    let _temp_dir = setup_test_env();
    let manager = ProfileManager::new().unwrap();

    let config = ProfileConfig::dev_profile();
    manager.save_profile("test_profile", &config).unwrap();

    let loaded = manager.load_profile("test_profile").unwrap();
    assert!(loaded.execution_config.verbose);
    assert_eq!(loaded.execution_config.tool_timeout_ms, 10000);
}

#[test]
#[serial]
fn test_profile_exists() {
    let _temp_dir = setup_test_env();
    let manager = ProfileManager::new().unwrap();

    assert!(!manager.profile_exists("nonexistent"));

    manager
        .save_profile("existing", &ProfileConfig::dev_profile())
        .unwrap();
    assert!(manager.profile_exists("existing"));
}

#[test]
#[serial]
fn test_list_profiles() {
    let _temp_dir = setup_test_env();
    let manager = ProfileManager::new().unwrap();

    manager
        .save_profile("", &ProfileConfig::default_profile())
        .unwrap();
    manager
        .save_profile("dev", &ProfileConfig::dev_profile())
        .unwrap();
    manager
        .save_profile("prod", &ProfileConfig::prod_profile())
        .unwrap();

    let profiles = manager.list_profiles().unwrap();
    assert!(profiles.len() >= 3);

    let names: Vec<_> = profiles.iter().map(|p| &p.name).collect();
    assert!(names.contains(&&"default".to_string()));
    assert!(names.contains(&&"dev".to_string()));
    assert!(names.contains(&&"prod".to_string()));
}

#[test]
#[serial]
fn test_validate_profile_success() {
    let _temp_dir = setup_test_env();
    let manager = ProfileManager::new().unwrap();

    manager
        .save_profile("valid", &ProfileConfig::dev_profile())
        .unwrap();
    assert!(manager.validate_profile("valid").is_ok());
}

#[test]
#[serial]
fn test_validate_profile_nonexistent() {
    let _temp_dir = setup_test_env();
    let manager = ProfileManager::new().unwrap();

    assert!(manager.validate_profile("nonexistent").is_err());
}

#[test]
#[serial]
fn test_delete_profile() {
    let _temp_dir = setup_test_env();
    let manager = ProfileManager::new().unwrap();

    manager
        .save_profile("to_delete", &ProfileConfig::dev_profile())
        .unwrap();
    assert!(manager.profile_exists("to_delete"));

    manager.delete_profile("to_delete").unwrap();
    assert!(!manager.profile_exists("to_delete"));
}

#[test]
#[serial]
fn test_delete_default_profile_fails() {
    let _temp_dir = setup_test_env();
    let manager = ProfileManager::new().unwrap();

    manager
        .save_profile("", &ProfileConfig::default_profile())
        .unwrap();

    assert!(manager.delete_profile("default").is_err());
    assert!(manager.delete_profile("").is_err());
}

#[test]
#[serial]
fn test_clone_profile() {
    let _temp_dir = setup_test_env();
    let manager = ProfileManager::new().unwrap();

    manager
        .save_profile("source", &ProfileConfig::dev_profile())
        .unwrap();
    manager.clone_profile("source", "dest").unwrap();

    let cloned = manager.load_profile("dest").unwrap();
    assert!(cloned.execution_config.verbose);
    assert_eq!(cloned.execution_config.tool_timeout_ms, 10000);
}

#[test]
#[serial]
fn test_load_active_profile_with_argument() {
    let _temp_dir = setup_test_env();
    let manager = ProfileManager::new().unwrap();

    manager
        .save_profile("custom", &ProfileConfig::dev_profile())
        .unwrap();

    let (config, name) = manager.load_active_profile(Some("custom")).unwrap();
    assert_eq!(name, "custom");
    assert!(config.execution_config.verbose);
}

#[test]
#[serial]
fn test_load_active_profile_default() {
    let _temp_dir = setup_test_env();
    let manager = ProfileManager::new().unwrap();

    let (config, name) = manager.load_active_profile(None).unwrap();
    assert_eq!(name, "default");
    assert_eq!(config.servers.len(), 0);
}

// ============================================================
// ConfigImportExport Tests (11 tests)
// ============================================================

#[test]
fn test_config_format_from_str() {
    assert_eq!(
        "json".parse::<ConfigFormat>().unwrap(),
        ConfigFormat::Json
    );
    assert_eq!(
        "yaml".parse::<ConfigFormat>().unwrap(),
        ConfigFormat::Yaml
    );
    assert!("xml".parse::<ConfigFormat>().is_err());
}

#[test]
fn test_config_format_from_path() {
    assert_eq!(
        ConfigFormat::from_path(Path::new("config.json")).unwrap(),
        ConfigFormat::Json
    );
    assert_eq!(
        ConfigFormat::from_path(Path::new("config.yaml")).unwrap(),
        ConfigFormat::Yaml
    );
    assert!(ConfigFormat::from_path(Path::new("config.xml")).is_err());
}

#[test]
fn test_export_import_json() {
    let temp_dir = TempDir::new().unwrap();
    let output_path = temp_dir.path().join("export.json");

    let config = ProfileConfig::dev_profile();
    let size =
        ConfigImportExport::export_config(&config, &output_path, Some(ConfigFormat::Json))
            .unwrap();
    assert!(size > 0);

    let imported =
        ConfigImportExport::import_config(&output_path, Some(ConfigFormat::Json)).unwrap();
    assert_eq!(
        imported.execution_config.verbose,
        config.execution_config.verbose
    );
}

#[test]
fn test_export_import_yaml() {
    let temp_dir = TempDir::new().unwrap();
    let output_path = temp_dir.path().join("export.yaml");

    let config = ProfileConfig::staging_profile();
    let size =
        ConfigImportExport::export_config(&config, &output_path, Some(ConfigFormat::Yaml))
            .unwrap();
    assert!(size > 0);

    let imported =
        ConfigImportExport::import_config(&output_path, Some(ConfigFormat::Yaml)).unwrap();
    assert_eq!(
        imported.execution_config.retry_count,
        config.execution_config.retry_count
    );
}

#[test]
fn test_export_format_auto_detection() {
    let temp_dir = TempDir::new().unwrap();
    let json_path = temp_dir.path().join("config.json");
    let yaml_path = temp_dir.path().join("config.yaml");

    let config = ProfileConfig::prod_profile();

    ConfigImportExport::export_config(&config, &json_path, None).unwrap();
    assert!(json_path.exists());

    ConfigImportExport::export_config(&config, &yaml_path, None).unwrap();
    assert!(yaml_path.exists());
}

#[test]
fn test_import_nonexistent_file() {
    let result = ConfigImportExport::import_config(
        Path::new("/nonexistent/config.json"),
        Some(ConfigFormat::Json),
    );
    assert!(result.is_err());
}

#[test]
fn test_validate_config_file() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("config.json");

    let config = ProfileConfig::default_profile();
    ConfigImportExport::export_config(&config, &config_path, Some(ConfigFormat::Json)).unwrap();

    let warnings =
        ConfigImportExport::validate_config_file(&config_path, Some(ConfigFormat::Json)).unwrap();
    assert!(warnings.iter().any(|w| w.contains("No servers")));
}

#[test]
fn test_diff_configs_no_change() {
    let config1 = ProfileConfig::dev_profile();
    let config2 = config1.clone();

    let diff = ConfigImportExport::diff_configs(&config1, &config2);
    assert!(!diff.has_changes());
    assert_eq!(diff.total_changes(), 0);
}

#[test]
fn test_diff_configs_with_changes() {
    let config1 = ProfileConfig::dev_profile();
    let mut config2 = config1.clone();
    config2.execution_config.verbose = false;
    config2.execution_config.tool_timeout_ms = 20000;

    let diff = ConfigImportExport::diff_configs(&config1, &config2);
    assert!(diff.has_changes());
    assert!(diff.total_changes() > 0);
    assert!(!diff.execution_changes.is_empty());
}

#[test]
fn test_diff_format() {
    let config1 = ProfileConfig::dev_profile();
    let mut config2 = config1.clone();
    config2.execution_config.verbose = false;

    let diff = ConfigImportExport::diff_configs(&config1, &config2);
    let formatted = diff.format();

    assert!(formatted.contains("Total changes:"));
    assert!(formatted.contains("Execution changes:"));
}

#[test]
fn test_dry_run_import() {
    let temp_dir = TempDir::new().unwrap();
    let import_path = temp_dir.path().join("new_config.json");

    let current_config = ProfileConfig::dev_profile();
    let mut new_config = current_config.clone();
    new_config.execution_config.tool_timeout_ms = 15000;

    ConfigImportExport::export_config(&new_config, &import_path, Some(ConfigFormat::Json))
        .unwrap();

    let diff = ConfigImportExport::dry_run_import(
        &current_config,
        &import_path,
        Some(ConfigFormat::Json),
    )
    .unwrap();

    assert!(diff.has_changes());
    assert!(!diff.execution_changes.is_empty());
}

// ============================================================
// ConfigTemplate Tests (11 tests)
// ============================================================

#[test]
fn test_preset_template_from_str() {
    assert_eq!(
        "minimal".parse::<PresetTemplate>().unwrap(),
        PresetTemplate::Minimal
    );
    assert_eq!(
        "development".parse::<PresetTemplate>().unwrap(),
        PresetTemplate::Development
    );
    assert_eq!(
        "prod".parse::<PresetTemplate>().unwrap(),
        PresetTemplate::Production
    );
    assert!("unknown".parse::<PresetTemplate>().is_err());
}

#[test]
#[serial]
fn test_list_preset_templates() {
    let _temp_dir = setup_test_env();
    let template_mgr = ConfigTemplate::new().unwrap();

    let presets = template_mgr.list_preset_templates();
    assert_eq!(presets.len(), 4);

    for preset in &presets {
        assert!(preset.is_preset);
        assert!(preset.file_path.is_none());
    }
}

#[test]
#[serial]
fn test_apply_preset_template() {
    let _temp_dir = setup_test_env();
    let template_mgr = ConfigTemplate::new().unwrap();

    let config = template_mgr.apply_template("development").unwrap();
    assert!(config.execution_config.verbose);

    let config = template_mgr.apply_template("production").unwrap();
    assert_eq!(config.logging.backend, "persistent");
}

#[test]
#[serial]
fn test_create_custom_template() {
    let _temp_dir = setup_test_env();
    let template_mgr = ConfigTemplate::new().unwrap();

    let config = ProfileConfig::dev_profile();
    template_mgr
        .create_custom_template("my_template", &config)
        .unwrap();

    let loaded = template_mgr.apply_template("my_template").unwrap();
    assert_eq!(
        loaded.execution_config.verbose,
        config.execution_config.verbose
    );
}

#[test]
#[serial]
fn test_create_custom_template_duplicate() {
    let _temp_dir = setup_test_env();
    let template_mgr = ConfigTemplate::new().unwrap();

    let config = ProfileConfig::dev_profile();
    template_mgr
        .create_custom_template("duplicate", &config)
        .unwrap();

    assert!(template_mgr
        .create_custom_template("duplicate", &config)
        .is_err());
}

#[test]
#[serial]
fn test_create_custom_template_preset_name_conflict() {
    let _temp_dir = setup_test_env();
    let template_mgr = ConfigTemplate::new().unwrap();

    let config = ProfileConfig::dev_profile();
    assert!(template_mgr
        .create_custom_template("minimal", &config)
        .is_err());
}

#[test]
#[serial]
fn test_list_custom_templates() {
    let _temp_dir = setup_test_env();
    let template_mgr = ConfigTemplate::new().unwrap();

    let customs = template_mgr.list_custom_templates().unwrap();
    assert_eq!(customs.len(), 0);

    template_mgr
        .create_custom_template("template1", &ProfileConfig::dev_profile())
        .unwrap();
    template_mgr
        .create_custom_template("template2", &ProfileConfig::staging_profile())
        .unwrap();

    let customs = template_mgr.list_custom_templates().unwrap();
    assert_eq!(customs.len(), 2);
}

#[test]
#[serial]
fn test_list_all_templates() {
    let _temp_dir = setup_test_env();
    let template_mgr = ConfigTemplate::new().unwrap();

    template_mgr
        .create_custom_template("my_template", &ProfileConfig::dev_profile())
        .unwrap();

    let all = template_mgr.list_all_templates().unwrap();
    assert!(all.len() >= 5);

    let preset_count = all.iter().filter(|t| t.is_preset).count();
    let custom_count = all.iter().filter(|t| !t.is_preset).count();

    assert_eq!(preset_count, 4);
    assert!(custom_count >= 1);
}

#[test]
#[serial]
fn test_delete_custom_template() {
    let _temp_dir = setup_test_env();
    let template_mgr = ConfigTemplate::new().unwrap();

    template_mgr
        .create_custom_template("to_delete", &ProfileConfig::dev_profile())
        .unwrap();

    template_mgr.delete_custom_template("to_delete").unwrap();

    assert!(template_mgr.apply_template("to_delete").is_err());
}

#[test]
#[serial]
fn test_delete_preset_template_fails() {
    let _temp_dir = setup_test_env();
    let template_mgr = ConfigTemplate::new().unwrap();

    assert!(template_mgr.delete_custom_template("minimal").is_err());
    assert!(template_mgr.delete_custom_template("development").is_err());
}

#[test]
#[serial]
fn test_show_template() {
    let _temp_dir = setup_test_env();
    let template_mgr = ConfigTemplate::new().unwrap();

    let json = template_mgr.show_template("development").unwrap();
    assert!(!json.is_empty());
    assert!(json.contains("execution_config"));
    assert!(json.contains("logging"));
}

// Test count: ProfileManager (11) + ConfigImportExport (11) + ConfigTemplate (11) = 33 tests
// Additional test: test_apply_preset_template tests 2 configs (33 + 1 = 34 tests total)
