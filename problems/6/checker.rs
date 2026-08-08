use std::env;
use std::fs;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::BufRead;
use std::io::BufReader;
use std::io::Write;
use std::process::exit;

const OK: i32 = 0;
const WRONG_ANSWER: i32 = 1;
const PRESENTATION_ERROR: i32 = 2;
const FAIL: i32 = 3;

fn die(code: i32, msg: &str) -> ! {
    eprintln!("{msg}");
    exit(code);
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 4 {
        die(FAIL, "Checker takes 4 arguments");
    }

    let input_path = &args[1];
    let output_path = &args[2];
    let answer_path = &args[3];

    let answer: Vec<i32> = fs::read_to_string(answer_path)
        .unwrap_or_else(|_| die(FAIL, "Cannot read answer"))
        .trim()
        .split_whitespace()
        .map(|x| {
            x.parse()
                .unwrap_or_else(|_| die(FAIL, "Cannot parse answer"))
        })
        .collect();

    let (mut k, t, x) = (answer[0], answer[1], answer[2]);

    let mut output_file = OpenOptions::new()
        .write(true)
        .truncate(true)
        .open(output_path)
        .unwrap_or_else(|_| die(FAIL, "Cannot open output"));
    writeln!(output_file, "{t}").unwrap_or_else(|_| die(FAIL, "Cannot write to output"));
    let input_file = File::open(input_path).unwrap_or_else(|_| die(FAIL, "Cannot open input"));
    let mut reader = BufReader::new(input_file);

    let mut s = String::new();
    while k > 0 {
        let n = reader
            .read_line(&mut s)
            .unwrap_or_else(|_| die(FAIL, "Cannot read input"));
        let input: Vec<&str> = s.trim().split(" ").collect();
        if n == 0 || input.len() < 2 {
            die(PRESENTATION_ERROR, "Incorrect input");
        }
        if input[0].trim() != "?" {
            die(PRESENTATION_ERROR, "Incorrect input");
        }
        let y: i32 = input[1]
            .trim()
            .parse()
            .unwrap_or_else(|_| die(PRESENTATION_ERROR, "Incorrect input"));
        if x == y {
            writeln!(output_file, "=").unwrap_or_else(|_| die(FAIL, "Cannot write to output"));
            die(OK, "Ok");
        } else if x < y {
            writeln!(output_file, "<").unwrap_or_else(|_| die(FAIL, "Cannot write to output"));
        } else {
            writeln!(output_file, ">").unwrap_or_else(|_| die(FAIL, "Cannot write to output"));
        }
        k -= 1;
        s.clear();
    }
    die(WRONG_ANSWER, "Too many operations");
}
