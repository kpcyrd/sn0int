extern crate sn0int;

use std::env;
use std::thread;
use std::time::Duration;
use sn0int::term::{SPINNERS, Spinner};

fn main() {
    let idx = env::args().skip(1).next().expect("Expected argv[1]");
    let idx = idx.parse::<usize>().expect("argv[1] is not a number");

    let mut s = Spinner::new(SPINNERS[idx], "Demo".to_string());

    for _ in 0..100 {
        thread::sleep(Duration::from_millis(100));
        s.tick();
    }

    s.finish("Done".to_string());
}
