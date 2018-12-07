use crate::errors::*;
use tar;
use libflate::gzip;
use std::io;
use std::fs::File;
use std::path::Path;


pub fn extract<R: io::Read, P: AsRef<Path>>(read: &mut R, filter: &str, target: P) -> Result<()> {
    let file = gzip::Decoder::new(read)?;
    let mut ar = tar::Archive::new(file);

    for entry in ar.entries()? {
        let mut entry = entry?;
        let file_name = {
            let path = entry.path()?;
            path.file_name()
                .ok_or_else(|| format_err!("Invalid path in archive"))?
                .to_str()
                .ok_or_else(|| format_err!("Filename is invalid utf8"))?
                .to_owned()
        };

        debug!("Found in archive: {:?}", file_name);

        if filter == file_name {
            debug!("Extracting to {:?}", target.as_ref());

            let mut target = File::create(target)?;
            let n = io::copy(&mut entry, &mut target)?;
            debug!("Wrote {:?} bytes", n);

            return Ok(());
        }

        debug!("Skipping file");
    }

    bail!("Nothing in archive matched filter")
}
