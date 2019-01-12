use std::thread;
use std::time::Duration;
use sn0int::term::{SPINNERS, Spinner, StackedSpinners};
use structopt::StructOpt;


#[derive(Debug, StructOpt)]
pub enum Args {
    #[structopt(name="single")]
    Single(Single),
    #[structopt(name="stacked")]
    Stacked(Stacked),
}

#[derive(Debug, StructOpt)]
pub struct Single {
    idx: usize,
    #[structopt(long="ticks", default_value="100")]
    ticks: usize,
}

impl Single {
    fn run(&self) {
        let mut s = Spinner::new(SPINNERS[self.idx], "Demo".to_string());

        for _ in 0..self.ticks {
            thread::sleep(Duration::from_millis(100));
            s.tick();
        }

        s.finish("Done".to_string());
    }
}

#[derive(Debug, StructOpt)]
pub struct Stacked {
}

impl Stacked {
    fn run(&self) {
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
}

fn main() {
    let args = Args::from_args();
    match args {
        Args::Single(args) => args.run(),
        Args::Stacked(args) => args.run(),
    }
}
