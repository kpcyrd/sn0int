use crate::errors::*;

use crate::blobs::Blob;
use crate::cmd::Cmd;
use crate::shell::Shell;
use crate::term;
use crate::worker;
use crate::models::*;
use std::collections::HashSet;
use structopt::StructOpt;


#[derive(Debug, StructOpt)]
pub struct Args {
    /// Verbose output
    #[structopt(short="v", long="verbose", parse(from_occurrences))]
    verbose: u64,
    /// Delete only dangling blobs
    #[structopt(long="gc")]
    gc: bool,
    /// Delete dangling and corrupted blobs
    #[structopt(long="gc-all")]
    gc_all: bool,
}

impl Cmd for Args {
    fn run(self, rl: &mut Shell) -> Result<()> {
        let blobs = rl.blobs();

        let hashset = worker::spawn_fn("Building reference set...", || {
            let mut hashset = HashSet::new();

            for x in rl.db().list::<Image>()? {
                hashset.insert(x.value);
            }

            Ok(hashset)
        }, true)?;

        for blob in blobs.list()? {

            let state = worker::spawn_fn(&blob, || {
                let blob = rl.blobs().load(&blob)?;

                // ensure content matches hash
                if Blob::hash(&blob.bytes) != blob.id {
                    Ok(State::Corrupted)
                // ensure blob is referenced by any table
                } else if !hashset.contains(&blob.id) {
                    Ok(State::Dangling)
                } else {
                    Ok(State::Valid)
                }
            }, true);

            match state {
                Ok(State::Valid) => {
                    if self.verbose > 0 {
                        term::info(&format!("{}... ok", blob));
                    }
                },
                Ok(State::Dangling) => {
                    term::warn(&format!("{}... dangling", blob));
                    if self.gc || self.gc_all {
                        blobs.delete(&blob)?;
                    }
                },
                Ok(State::Corrupted) => {
                    term::error(&format!("{}... corrupted", blob));
                    if self.gc_all {
                        blobs.delete(&blob)?;
                    }
                },
                Err(err) => {
                    term::error(&format!("{}... {}", blob, err));
                },
            }
        }

        Ok(())
    }
}

enum State {
    Valid,
    Dangling,
    Corrupted,
}
