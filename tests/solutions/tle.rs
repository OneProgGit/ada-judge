use std::io::stdin;
use std::thread::sleep;
use std::time::Duration;

fn main() {
    let mut s = String::new();
    stdin().read_line(&mut s).unwrap();

    let a: Vec<i32> = s.trim().split(" ").map(|x| x.parse().unwrap()).collect();

    sleep(Duration::from_secs(2));

    println!("{}", a[0] + a[1]);
}
