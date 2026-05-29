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
