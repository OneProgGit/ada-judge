use solutions_judger::enums::AdaJudgeVerdict;
use std::{fs, process::Command};

fn compile(solution_name: &str, env_path: String) {
    Command::new("rustc")
        .args([
            &format!("solutions/{}.rs", solution_name),
            "-o",
            &(env_path.clone() + "/run"),
        ])
        .spawn()
        .unwrap()
        .wait()
        .unwrap();
}

fn test_usual(solution_name: &str, with_deps: bool, verdict: AdaJudgeVerdict) {
    let env_path = if with_deps {
        format!("env_{}_with_deps", solution_name)
    } else {
        format!("env_{}_no_deps", solution_name)
    };

    compile(solution_name, env_path.clone());

    let problem_id = if with_deps { 2 } else { 1 };
    let res = solutions_judger::test(format!("problems/{problem_id}"), &env_path).unwrap();

    fs::remove_dir_all(&env_path).unwrap();
    fs::create_dir(&env_path).unwrap();

    assert_eq!(res.groups_result[0].verdict, verdict);
    assert_eq!(res.groups_result[0].score, 0);
    if verdict != AdaJudgeVerdict::Ok {
        assert_eq!(res.total_score, 0);
        assert_eq!(res.groups_result[1].score, 0);
        if with_deps {
            assert_eq!(res.groups_result[1].verdict, AdaJudgeVerdict::Skipped);
        } else {
            assert_eq!(res.groups_result[1].verdict, verdict);
        }
    } else {
        assert_eq!(res.total_score, 100);
        assert_eq!(res.groups_result[1].score, 100);
        assert_eq!(res.groups_result[1].verdict, verdict);
    }
}

fn test_incorrect_deps(solution_name: &str) {
    let env_path = format!("env_{solution_name}_incorrect_deps");
    compile(solution_name, env_path.clone());
    solutions_judger::test("problems/3", env_path.clone()).unwrap_err();

    fs::remove_dir_all(&env_path).unwrap();
    fs::create_dir(&env_path).unwrap();
}

#[test]
fn test_ok_no_deps() {
    test_usual("ok", false, AdaJudgeVerdict::Ok);
}

#[test]
fn test_wa_no_deps() {
    test_usual("wa", false, AdaJudgeVerdict::WrongAnswer);
}

#[test]
fn test_tle_no_deps() {
    test_usual("tle", false, AdaJudgeVerdict::TimeLimitExceeded);
}

#[test]
fn test_mle_no_deps() {
    test_usual("mle", false, AdaJudgeVerdict::MemoryLimitExceeded);
}

#[test]
fn test_re_no_deps() {
    test_usual("re", false, AdaJudgeVerdict::RuntimeError);
}

#[test]
fn test_ok_with_deps() {
    test_usual("ok", true, AdaJudgeVerdict::Ok);
}

#[test]
fn test_wa_with_deps() {
    test_usual("wa", true, AdaJudgeVerdict::WrongAnswer);
}

#[test]
fn test_tle_with_deps() {
    test_usual("tle", true, AdaJudgeVerdict::TimeLimitExceeded);
}

#[test]
fn test_mle_with_deps() {
    test_usual("mle", true, AdaJudgeVerdict::MemoryLimitExceeded);
}

#[test]
fn test_re_with_deps() {
    test_usual("re", true, AdaJudgeVerdict::RuntimeError);
}

#[test]
fn test_ok_incorrect_deps() {
    test_incorrect_deps("ok");
}

#[test]
fn test_wa_incorrect_deps() {
    test_incorrect_deps("wa");
}

#[test]
fn test_tle_incorrect_deps() {
    test_incorrect_deps("tle");
}

#[test]
fn test_mle_incorrect_deps() {
    test_incorrect_deps("mle");
}

#[test]
fn test_re_incorrect_deps() {
    test_incorrect_deps("re");
}
