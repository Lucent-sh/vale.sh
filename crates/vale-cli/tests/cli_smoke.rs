use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn vale_help_succeeds() {
    Command::cargo_bin("vale")
        .unwrap()
        .arg("--help")
        .assert()
        .success();
}

#[test]
fn vale_no_subcommand_shows_help() {
    Command::cargo_bin("vale")
        .unwrap()
        .assert()
        .success()
        .stdout(predicate::str::contains("vale"));
}

#[test]
fn doctor_json_output() {
    Command::cargo_bin("vale")
        .unwrap()
        .args(["doctor", "--output", "json"])
        .assert()
        .success()
        .stdout(predicate::str::contains("vale_version"));
}

#[test]
fn strategy_list() {
    Command::cargo_bin("vale")
        .unwrap()
        .args(["strategy", "list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("buy_and_hold"));
}
