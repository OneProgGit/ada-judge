use std::io::stdin;

fn main() {
    let mut s = String::new();
    stdin().read_line(&mut s).unwrap();

    let stage: Vec<i32> = s.trim().split(" ").map(|x| x.parse().unwrap()).collect();
    let stage: i32 = stage[0];

    s.clear();
    stdin().read_line(&mut s).unwrap();

    let x: Vec<i32> = s.trim().split(" ").map(|x| x.parse().unwrap()).collect();
    let x: i32 = x[0];

    if stage == 0 {
        println!("{}", x + 1);
    } else {
        println!("{}", x - 1);
    }
}
