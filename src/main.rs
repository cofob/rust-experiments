use std::io::Write;
use std::str::FromStr;

fn gcd(mut n: u64, mut m: u64) -> u64 {
    assert!(n != 0 && m != 0);
    while m != 0 {
        if m < n {
            let t = m;
            m = n;
            n = t;
        }
        m = m % n;
    }
    n
}

#[test]
fn test_gcd() {
    assert_eq!(gcd(14, 15), 1);

    assert_eq!(gcd(2 * 3 * 5 * 11 * 17, 3 * 7 * 11 * 13 * 19), 3 * 11);
}

fn main() {
    if std::env::args().len() == 1 {
        writeln!(
            std::io::stderr(),
            "Usage: {} [NUMBER1] [NUMBER2]",
            std::env::args().nth(0).unwrap()
        )
        .unwrap();
        std::process::exit(1);
    }

    if std::env::args().len() != 3 {
        writeln!(std::io::stderr(), "Error: You must specify two numbers.").unwrap();
        std::process::exit(1);
    }

    let numbers: Vec<u64> = std::env::args()
        .skip(1)
        .map(|arg| u64::from_str(&arg).expect("error parsing argument"))
        .collect();

    println!("Output: {}", gcd(numbers[0], numbers[1]));
}
