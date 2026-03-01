use std::io::stdin;

fn main() {
    let mut s = String::new();
    stdin().read_line(&mut s).unwrap();

    let a: Vec<i32> = s.trim().split(" ").map(|x| x.parse().unwrap()).collect();

    println!("{}", a[0] + a[1]);
}
