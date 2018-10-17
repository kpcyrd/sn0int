use errors::*;
#[cfg(target_os = "linux")]
use caps::{self, CapSet};
use nix;

#[cfg(target_os = "linux")]
pub mod seccomp;

static CHROOT: &str = "/var/empty";


#[cfg(target_os = "linux")]
/// Drop all privileges that are only needed to setup the sandbox
pub fn fasten_seatbelt() -> Result<()> {
    info!("Dropping all capabilities");
    caps::clear(None, CapSet::Effective)
        .map_err(|_| format_err!("Failed to clear effective capability set"))?;
    caps::clear(None, CapSet::Permitted)
        .map_err(|_| format_err!("Failed to clear permitted capability set"))?;
    Ok(())
}

#[cfg(not(target_os = "linux"))]
pub fn fasten_seatbelt() -> Result<()> {
    Ok(())
}

#[cfg(target_os = "linux")]
pub fn init() -> Result<()> {
    if let Err(err) = nix::unistd::chroot(CHROOT) {
        // TODO: add setting to make this a hard fail
        warn!("Failed to chroot: {:?}", err);
    } else {
        nix::unistd::chdir("/")?;
        info!("Successful chroot to {:?}", CHROOT);
    }

    fasten_seatbelt()?;

    #[cfg(target_os = "linux")]
    seccomp::init()?;

    Ok(())
}

#[cfg(not(target_os = "linux"))]
pub fn init() -> Result<()> {
    Ok(())
}
