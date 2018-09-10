extern crate rustyline;
extern crate env_logger;
extern crate rand;
extern crate colored;
extern crate failure;
extern crate shellwords;
#[macro_use] extern crate lazy_static;

pub mod engine;
pub mod shell;
pub mod term;
pub mod worker;


fn main() {
    env_logger::init();

    shell::run();
}
