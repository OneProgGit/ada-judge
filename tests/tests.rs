use ada_judge::verdicts::Verdict;
use std::process::Command;
use std::{fs, path::PathBuf};

fn compile(solution_name: &str) {
    Command::new("rustc")
        .args([
            &format!("tests/solutions/{}.rs", solution_name),
            "-o",
            &format!("tests/env_{}/run", solution_name),
        ])
        .spawn()
        .unwrap()
        .wait()
        .unwrap();
}

#[test]
fn test_ok() {
    compile("ok");
    let res = ada_judge::test(
        PathBuf::from("tests/problems/1"),
        PathBuf::from("tests/env_ok"),
    )
    .unwrap();

    fs::remove_dir_all("tests/env_ok").unwrap();
    fs::create_dir("tests/env_ok").unwrap();

    assert_eq!(res.groups_result[0].verdict, Verdict::Ok);
    assert_eq!(res.groups_result[1].verdict, Verdict::Ok);
    assert_eq!(res.total_score, 100);
}

#[test]
fn test_wa() {
    compile("wa");
    let res = ada_judge::test(
        PathBuf::from("tests/problems/1"),
        PathBuf::from("tests/env_wa"),
    )
    .unwrap();

    fs::remove_dir_all("tests/env_wa").unwrap();
    fs::create_dir("tests/env_wa").unwrap();

    assert_eq!(res.groups_result[0].verdict, Verdict::WrongAnswer);
    assert_eq!(res.groups_result[1].verdict, Verdict::WrongAnswer);
    assert_eq!(res.total_score, 0);
}

#[test]
fn test_tle() {
    compile("tle");
    let res = ada_judge::test(
        PathBuf::from("tests/problems/1"),
        PathBuf::from("tests/env_tle"),
    )
    .unwrap();

    fs::remove_dir_all("tests/env_tle").unwrap();
    fs::create_dir("tests/env_tle").unwrap();

    assert_eq!(res.groups_result[0].verdict, Verdict::TimeLimitExceeded);
    assert_eq!(res.groups_result[1].verdict, Verdict::TimeLimitExceeded);
    assert_eq!(res.total_score, 0);
}

#[test]
fn test_mle() {
    compile("mle");
    let res = ada_judge::test(
        PathBuf::from("tests/problems/1"),
        PathBuf::from("tests/env_mle"),
    )
    .unwrap();

    fs::remove_dir_all("tests/env_mle").unwrap();
    fs::create_dir("tests/env_mle").unwrap();

    assert_eq!(res.groups_result[0].verdict, Verdict::MemoryLimitExceeded);
    assert_eq!(res.groups_result[1].verdict, Verdict::MemoryLimitExceeded);
    assert_eq!(res.total_score, 0);
}

#[test]
fn test_re() {
    compile("re");
    let res = ada_judge::test(
        PathBuf::from("tests/problems/1"),
        PathBuf::from("tests/env_re"),
    )
    .unwrap();

    fs::remove_dir_all("tests/env_re").unwrap();
    fs::create_dir("tests/env_re").unwrap();

    assert_eq!(res.groups_result[0].verdict, Verdict::RuntimeError);
    assert_eq!(res.groups_result[1].verdict, Verdict::RuntimeError);
    assert_eq!(res.total_score, 0);
}
