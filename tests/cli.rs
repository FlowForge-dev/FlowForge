use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::tempdir;

fn forge(home: &std::path::Path) -> Command {
    let mut cmd = Command::cargo_bin("forge").expect("forge binary");
    cmd.env("FORGEFLOW_HOME", home);
    cmd
}

#[test]
fn provider_list_runs_without_config() {
    let home = tempdir().unwrap();
    forge(home.path())
        .arg("provider")
        .arg("list")
        .assert()
        .success()
        .stdout(predicate::str::contains("Supported Providers"));
    assert!(home.path().join("config.toml").exists());
    assert!(home.path().join("patterns").is_dir());
    assert!(home.path().join("workflows").is_dir());
    assert!(home.path().join("plugins").is_dir());
}

#[test]
fn init_creates_everything() {
    let home = tempdir().unwrap();
    forge(home.path())
        .arg("init")
        .assert()
        .success()
        .stdout(predicate::str::contains("ForgeFlow is ready"));
    assert!(home.path().join("config.toml").exists());
    assert!(home.path().join("patterns").is_dir());
    assert!(home.path().join("workflows").is_dir());
    assert!(home.path().join("plugins").is_dir());
}

#[test]
fn config_path_prints_real_config_file() {
    let home = tempdir().unwrap();
    forge(home.path())
        .arg("config")
        .arg("path")
        .assert()
        .success()
        .stdout(predicate::str::contains("config.toml"));
    assert!(home.path().join("config.toml").exists());
}

#[test]
fn config_show_prints_toml() {
    let home = tempdir().unwrap();
    forge(home.path())
        .arg("config")
        .arg("show")
        .assert()
        .success()
        .stdout(predicate::str::contains("provider = \"openai\""));
}

#[test]
fn provider_configure_openrouter_sets_key_and_model() {
    let home = tempdir().unwrap();
    forge(home.path())
        .arg("provider")
        .arg("configure")
        .arg("openrouter")
        .arg("--api-key")
        .arg("sk-or-test-secret")
        .assert()
        .success()
        .stdout(predicate::str::contains("Configured provider openrouter"));

    forge(home.path())
        .arg("provider")
        .arg("info")
        .arg("openrouter")
        .assert()
        .success()
        .stdout(predicate::str::contains("openrouter/free"))
        .stdout(predicate::str::contains("api_key     yes"));
}

#[test]
fn config_show_redacts_direct_api_keys() {
    let home = tempdir().unwrap();
    forge(home.path())
        .arg("provider")
        .arg("configure")
        .arg("openrouter")
        .arg("--api-key")
        .arg("sk-or-test-secret")
        .assert()
        .success();

    forge(home.path())
        .arg("config")
        .arg("show")
        .assert()
        .success()
        .stdout(predicate::str::contains("sk-or-test-secret").not())
        .stdout(predicate::str::contains("<redacted>"));
}

#[test]
fn shell_tool_requires_explicit_opt_in() {
    let home = tempdir().unwrap();
    forge(home.path())
        .arg("tool")
        .arg("shell")
        .arg("echo")
        .arg("hello")
        .assert()
        .failure()
        .stderr(predicate::str::contains("shell tool is disabled"));
}

#[test]
fn plugin_scaffold_rejects_path_traversal() {
    let home = tempdir().unwrap();
    forge(home.path())
        .arg("plugin")
        .arg("scaffold")
        .arg("../bad")
        .assert()
        .failure()
        .stderr(predicate::str::contains("invalid plugin name"));
}

#[test]
fn write_tool_refuses_paths_outside_workspace_by_default() {
    let home = tempdir().unwrap();
    let workspace = tempdir().unwrap();
    let outside_dir = tempdir().unwrap();
    let outside = outside_dir.path().join("outside.txt");

    forge(home.path())
        .current_dir(workspace.path())
        .arg("tool")
        .arg("write")
        .arg(outside)
        .arg("nope")
        .assert()
        .failure()
        .stderr(predicate::str::contains("refusing to write outside"));
}
