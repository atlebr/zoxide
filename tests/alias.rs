use assert_cmd::Command;

#[test]
fn alias_add_list_jump_rm() {
    let tempdir = tempfile::tempdir().unwrap();
    let data_dir = tempdir.path().to_str().unwrap();

    let cwd = std::env::current_dir().unwrap();
    let cwd_str = cwd.to_str().unwrap();

    Command::cargo_bin("zoxide")
        .unwrap()
        .env("_ZO_DATA_DIR", data_dir)
        .args(["alias", "add", "proj", cwd_str])
        .assert()
        .success();

    let list = Command::cargo_bin("zoxide")
        .unwrap()
        .env("_ZO_DATA_DIR", data_dir)
        .args(["alias", "list"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let list = String::from_utf8(list).unwrap();
    assert!(list.contains("proj\t"));

    let jump = Command::cargo_bin("zoxide")
        .unwrap()
        .env("_ZO_DATA_DIR", data_dir)
        .args(["alias", "jump", "proj"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let jump = String::from_utf8(jump).unwrap();
    assert!(jump.contains(cwd_str));

    Command::cargo_bin("zoxide")
        .unwrap()
        .env("_ZO_DATA_DIR", data_dir)
        .args(["alias", "rm", "proj"])
        .assert()
        .success();
}

#[test]
fn alias_jump_missing_returns_code_1_and_warning() {
    let tempdir = tempfile::tempdir().unwrap();
    let data_dir = tempdir.path().to_str().unwrap();

    let err = Command::cargo_bin("zoxide")
        .unwrap()
        .env("_ZO_DATA_DIR", data_dir)
        .args(["alias", "jump", "missing"])
        .assert()
        .code(1)
        .get_output()
        .stderr
        .clone();
    let err = String::from_utf8(err).unwrap();
    assert!(err.contains("warning: alias \"missing\" not found; using zoxide match"));
}

#[test]
fn alias_list_complete_outputs_names() {
    let tempdir = tempfile::tempdir().unwrap();
    let data_dir = tempdir.path().to_str().unwrap();

    let cwd = std::env::current_dir().unwrap();
    let cwd_str = cwd.to_str().unwrap();

    Command::cargo_bin("zoxide")
        .unwrap()
        .env("_ZO_DATA_DIR", data_dir)
        .args(["alias", "add", "alpha", cwd_str])
        .assert()
        .success();

    Command::cargo_bin("zoxide")
        .unwrap()
        .env("_ZO_DATA_DIR", data_dir)
        .args(["alias", "add", "beta", cwd_str])
        .assert()
        .success();

    let complete = Command::cargo_bin("zoxide")
        .unwrap()
        .env("_ZO_DATA_DIR", data_dir)
        .args(["alias", "list-complete"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let complete = String::from_utf8(complete).unwrap();
    assert!(complete.contains("alpha"));
    assert!(complete.contains("beta"));
}
