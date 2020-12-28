use crate::errors::*;
#[cfg(target_os = "linux")]
use caps::{self, CapSet};

#[cfg(target_os = "openbsd")]
use pledge::pledge;
#[cfg(target_os = "openbsd")]
use unveil::unveil;

#[cfg(target_os = "linux")]
pub mod seccomp;

#[cfg(target_os = "linux")]
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

pub fn init() -> Result<()> {
    #[cfg(target_os = "linux")]
    init_linux()?;

    #[cfg(target_os = "openbsd")]
    init_openbsd()?;

    Ok(())
}

#[cfg(target_os = "linux")]
pub fn init_linux() -> Result<()> {
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

#[cfg(target_os = "openbsd")]
pub fn init_openbsd() -> Result<()> {
    unveil("/etc/resolv.conf", "r")
        .map_err(|_| format_err!("Failed to call unveil"))?;

    unveil("/dev/urandom", "r")
        .map_err(|_| format_err!("Failed to call unveil"))?;

    // disable further unveil calls
    unveil("", "")
        .map_err(|_| format_err!("Failed to call unveil"))?;

    pledge![Stdio Rpath Dns Inet,]?;

    Ok(())
}
