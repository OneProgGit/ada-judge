use ada_judge::verdicts::Verdict;
use std::{fs, process::Command};

fn test(solution_name: &str, with_deps: bool, verdict: Verdict) {
    let env_path = if with_deps {
        format!("tests/env_{}_with_deps", solution_name)
    } else {
        format!("tests/env_{}_no_deps", solution_name)
    };

    Command::new("rustc")
        .args([
            &format!("tests/solutions/{}.rs", solution_name),
            "-o",
            &(env_path.clone() + "/run"),
        ])
        .spawn()
        .unwrap()
        .wait()
        .unwrap();

    let problem_id = if with_deps { 2 } else { 1 };
    let res = ada_judge::test(format!("tests/problems/{problem_id}"), &env_path).unwrap();

    fs::remove_dir_all(&env_path).unwrap();
    fs::create_dir(&env_path).unwrap();

    assert_eq!(res.groups_result[0].verdict, verdict);
    assert_eq!(res.groups_result[0].score, 0);
    if verdict != Verdict::Ok {
        assert_eq!(res.total_score, 0);
        assert_eq!(res.groups_result[1].score, 0);
        if with_deps {
            assert_eq!(res.groups_result[1].verdict, Verdict::Skipped);
        } else {
            assert_eq!(res.groups_result[1].verdict, verdict);
        }
    } else {
        assert_eq!(res.total_score, 100);
        assert_eq!(res.groups_result[1].score, 100);
        assert_eq!(res.groups_result[1].verdict, verdict);
    }
}

#[test]
fn test_ok_no_deps() {
    test("ok", false, Verdict::Ok);
}

#[test]
fn test_wa_no_deps() {
    test("wa", false, Verdict::WrongAnswer);
}

#[test]
fn test_tle_no_deps() {
    test("tle", false, Verdict::TimeLimitExceeded);
}

#[test]
fn test_mle_no_deps() {
    test("mle", false, Verdict::MemoryLimitExceeded);
}

#[test]
fn test_re_no_deps() {
    test("re", false, Verdict::RuntimeError);
}

#[test]
fn test_ok_with_deps() {
    test("ok", true, Verdict::Ok);
}

#[test]
fn test_wa_with_deps() {
    test("wa", true, Verdict::WrongAnswer);
}

#[test]
fn test_tle_with_deps() {
    test("tle", true, Verdict::TimeLimitExceeded);
}

#[test]
fn test_mle_with_deps() {
    test("mle", true, Verdict::MemoryLimitExceeded);
}

#[test]
fn test_re_with_deps() {
    test("re", true, Verdict::RuntimeError);
}
