// TODO: It seems rustup is not installed in the docker image provided by cross.
#![cfg(not(target_env = "musl"))]
#![warn(rust_2018_idioms)]

mod auxiliary;

use anyhow::Context as _;
use auxiliary::{cargo_llvm_cov, test_report, CommandExt};
use fs_err as fs;

fn test_set() -> Vec<(&'static str, &'static [&'static str])> {
    vec![
        ("txt", &["--text"]),
        ("hide-instantiations.txt", &["--text", "--hide-instantiations"]),
        ("summary.txt", &[]),
        ("json", &["--json", "--summary-only"]),
        ("full.json", &["--json"]),
        ("lcov.info", &["--lcov", "--summary-only"]),
    ]
}

fn run(model: &str, name: &str, args: &[&str]) {
    let id = format!("{}/{}", model, name);
    for (extension, args2) in test_set() {
        test_report(model, name, extension, [args, args2].concat()).context(id.clone()).unwrap();
    }
}

// TODO:
// - add tests for non-crates.io dependencies

#[test]
fn real1() {
    run("real1", "workspace_root", &[]);
    run("real1", "all", &["--all"]);
    run("real1", "manifest_path", &["--manifest-path", "member1/member2/Cargo.toml"]);
    run("real1", "package1", &["--package", "member2"]);
    run("real1", "exclude", &["--all", "--exclude", "crate1"]);
}

#[test]
fn virtual1() {
    run("virtual1", "workspace_root", &[]);
    run("virtual1", "package1", &["--package", "member1"]);
    run("virtual1", "package2", &["--package", "member1", "--package", "member2"]);
    run("virtual1", "package3", &["--package", "member2"]);
    run("virtual1", "package4", &["--package", "member3"]);
    run("virtual1", "package5", &["--package", "member4"]);
    run("virtual1", "package6", &["--package", "member3", "--package", "member4"]);
    run("virtual1", "exclude", &["--workspace", "--exclude", "member2"]);
}

#[test]
fn no_test() {
    // TODO: we should fix this: https://github.com/taiki-e/cargo-llvm-cov/issues/21
    run("no_test", "no_test", &[]);
}

#[test]
fn bin_crate() {
    run("bin_crate", "bin_crate", &[]);
}

#[test]
fn instantiations() {
    // TODO: fix https://github.com/taiki-e/cargo-llvm-cov/issues/43
    run("instantiations", "instantiations", &[]);
}

#[test]
fn cargo_config() {
    run("cargo_config", "cargo_config", &[]);
    run("cargo_config_toml", "cargo_config_toml", &[]);
}

#[test]
fn merge() {
    let model = "merge";
    let output_dir = auxiliary::FIXTURES_PATH.join("coverage-reports").join(model);
    fs::create_dir_all(&output_dir).unwrap();
    for (extension, args) in test_set() {
        let workspace_root = auxiliary::test_project(model, model).unwrap();
        let output_path = &output_dir.join(model).with_extension(extension);
        cargo_llvm_cov()
            .args(["--color", "never", "--no-report", "--features", "a"])
            .current_dir(workspace_root.path())
            .assert_success();
        cargo_llvm_cov()
            .args(["--color", "never", "--no-report", "--features", "b"])
            .current_dir(workspace_root.path())
            .assert_success();
        cargo_llvm_cov()
            .args(["--color", "never", "--no-run", "--output-path"])
            .arg(output_path)
            .args(args)
            .current_dir(workspace_root.path())
            .assert_success();

        auxiliary::normalize_output(output_path, args).unwrap();
        auxiliary::assert_output(output_path).unwrap();
    }
}
