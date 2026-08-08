use std::io::stdin;

fn main() {
    let mut s = String::new();
    stdin().read_line(&mut s).unwrap();

    let n: Vec<i32> = s.trim().split(" ").map(|x| x.parse().unwrap()).collect();
    let n: i32 = n[0];
    let mut l = 0;
    let mut r = n + 1;
    loop {
        let m = (l + r) / 2;
        println!("? {m}");
        s.clear();
        stdin().read_line(&mut s).unwrap();
        if s.trim() == "<" {
            r = m;
        } else if s.trim() == ">" {
            l = m;
        } else {
            break;
        }
    }
}
