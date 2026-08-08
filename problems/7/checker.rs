use std::env;
use std::fs;
use std::fs::OpenOptions;
use std::io::Write;
use std::process::exit;

const OK: i32 = 0;
const WRONG_ANSWER: i32 = 1;
const _PRESENTATION_ERROR: i32 = 2;
const FAIL: i32 = 3;

fn die(code: i32, msg: &str) -> ! {
    eprintln!("{msg}");
    exit(code);
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 5 {
        die(FAIL, "Checker takes 5 arguments");
    }

    let input_path = &args[2];
    let output_path = &args[1];
    let answer_path = &args[3];
    let stage: i32 = args[4]
        .parse()
        .unwrap_or_else(|_| die(FAIL, "Cannot parse stage"));

    let mut input_file = OpenOptions::new()
        .write(true)
        .truncate(true)
        .open(input_path)
        .unwrap_or_else(|_| die(FAIL, "Cannot open input"));
    let output =
        fs::read_to_string(output_path).unwrap_or_else(|_| die(WRONG_ANSWER, "Cannot read output"));
    let answer =
        fs::read_to_string(answer_path).unwrap_or_else(|_| die(FAIL, "Cannot read answer"));
    if stage == 0 {
        writeln!(&mut input_file, "0\n{answer}")
            .unwrap_or_else(|_| die(FAIL, "Cannot write to output"));
    } else if stage == 1 {
        let x: i32 = answer
            .trim()
            .parse()
            .unwrap_or_else(|_| die(FAIL, "Cannot parse answer"));
        let y: i32 = output
            .trim()
            .parse()
            .unwrap_or_else(|_| die(WRONG_ANSWER, "Cannot parse output"));
        if y != x + 1 {
            die(WRONG_ANSWER, "Not x + 1");
        }
        writeln!(&mut input_file, "1\n{y}").unwrap_or_else(|_| die(FAIL, "Cannot write to output"));
    } else {
        let x: i32 = answer
            .trim()
            .parse()
            .unwrap_or_else(|_| die(FAIL, "Cannot parse answer"));
        let y: i32 = output
            .trim()
            .parse()
            .unwrap_or_else(|_| die(WRONG_ANSWER, "Cannot parse output"));
        if y != x {
            die(WRONG_ANSWER, "Not x");
        }
        die(OK, "Ok");
    }
}
