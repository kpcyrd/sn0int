use crate::errors::*;

use crate::shell::Shell;


#[inline]
fn help(name: &str, descr: &str) {
    println!("    \x1b[32m{:13}\x1b[0m {}", name, descr);
}

pub fn run(_rl: &mut Shell, _args: &[String]) -> Result<()> {

    println!("\n\x1b[33mCOMMANDS:\x1b[0m");
    help("add",         "Add new entities to the database");
    help("autonoscope", "Manage rules to automatically remove entities from scope");
    help("autoscope",   "Manage rules to automatically add entities to scope");
    help("delete",      "Delete entities from the database");
    help("keyring",     "Manage saved credentials");
    help("pkg",         "Manage installed modules");
    help("noscope",     "Exclude entities from scope");
    help("run",         "Run the currently selected module");
    help("scope",       "Include entities in the scope again");
    help("select",      "Select entities from the database");
    help("stats",       "Show statistics about your current workspace");
    help("target",      "Preview targeted entities or narrow them down");
    help("use",         "Select a module");
    help("workspace",   "Switch to a different workspace");
    help("help",        "Prints this message");
    println!("\nRun <command> -h for more help.\n");

    Ok(())
}
