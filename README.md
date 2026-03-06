# ada-judge

> Note: all AI-made PR's  are banned from
> this project.

**To use it, you need docker to be installed!!!**

### Checker API:
```
(input: path, output: path, answer: path) -> Verdict
```

### Checker's verdicts:
```
0 - OK
1 - WrongAnswer
2 - PresentationError
3 - Fail
```

### Ada Judge's verdict is an enum:
```
Ok,
CompilationError,
RuntimeError,
TimeLimitExceeded,
MemoryLimitExceeded,
SecurityError,
WrongAnswer,
PresentationError,
Skipped,
```

### Getting started
You just need to build a docker container (make sure you're in the root directory):
```bash
docker build -t sandbox-runner .
```
Then, you can use ada-judge as a lib like that:
```rust
use ada_judge::{self, verdicts::Verdict};
use std::path::PathBuf;

fn print_verdict(verdict: Verdict) {
    match verdict {
        Verdict::Ok => print!("OK"),
        Verdict::CompilationError => print!("CE"),
        Verdict::RuntimeError => print!("RE"),
        Verdict::TimeLimitExceeded => print!("TLE"),
        Verdict::MemoryLimitExceeded => print!("MLE"),
        Verdict::SecurityError => print!("SE"),
        Verdict::WrongAnswer => print!("WA"),
        Verdict::PresentationError => print!("PE"),
        Verdict::Skipped => print!("SK"),
    }
}

fn main() {
    let problem_path = PathBuf::from("/home/leonid/Desktop/Projects/ada-judge/problems/1");
    let run_path = PathBuf::from("/home/leonid/Desktop/Projects/test-1");

    match ada_judge::test(problem_path, run_path) {
        Ok(verdicts) => {
            for res in verdicts {
                print_verdict(res.verdict);
                println!(": {}", res.test);
                println!("checker msg: \"{}\"", res.checker_msg.trim());
            }
        }
        Err(err) => {
            println!("{err}");
        }
    }
}
```