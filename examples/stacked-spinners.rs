extern crate sn0int;

use sn0int::term::StackedSpinners;
use std::thread;
use std::time::Duration;

fn main() {
    let mut stack = StackedSpinners::new();
    stack.add("1".into(), String::from("spinner1"));
    stack.add("2".into(), String::from("spinner2"));
    stack.add("3".into(), String::from("spinner3"));

    for x in 1..=3 {
        for _ in 0..50 {
            thread::sleep(Duration::from_millis(100));
            stack.tick();
        }
        // stack.log("ohai");
        stack.remove(&x.to_string());
    }

    stack.clear();

    // stack.finish("Done".to_string());
}
