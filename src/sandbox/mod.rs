use errors::*;
use caps::{self, CapSet};
use nix;

pub mod seccomp;

static CHROOT: &str = "/var/empty";


/// Drop all privileges that are only needed to setup the sandbox
pub fn fasten_seatbelt() -> Result<()> {
    info!("Dropping all capabilities");
    caps::clear(None, CapSet::Effective)
        .map_err(|_| format_err!("Failed to clear effective capability set"))?;
    caps::clear(None, CapSet::Permitted)
        .map_err(|_| format_err!("Failed to clear permitted capability set"))?;
    Ok(())
}

pub fn init() -> Result<()> {
    if let Err(err) = nix::unistd::chroot(CHROOT) {
        // TODO: add setting to make this a hard fail
        warn!("Failed to chroot: {:?}", err);
    } else {
        nix::unistd::chdir("/")?;
        info!("Successful chroot to {:?}", CHROOT);
    }
    fasten_seatbelt()?;

    seccomp::init()?;

    Ok(())
}
