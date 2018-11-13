extern crate sn0int;

use sn0int::term::StackedSpinners;
use std::thread;
use std::time::Duration;

fn main() {
    let mut stack = StackedSpinners::new();
    stack.add(String::from("spinner1"));
    stack.add(String::from("spinner2"));
    stack.add(String::from("spinner3"));

    while !stack.is_empty() {
        for _ in 0..50 {
            thread::sleep(Duration::from_millis(100));
            stack.tick();
        }
        stack.log("ohai");
        // stack.remove(0);
    }

    // stack.finish("Done".to_string());
}
