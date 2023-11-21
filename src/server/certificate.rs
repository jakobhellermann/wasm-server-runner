use std::io::ErrorKind;
use std::path::Path;

use crate::Result;
use directories::ProjectDirs;

pub struct Certificate {
    pub certificate: Vec<u8>,
    pub private_key: Vec<u8>,
}

pub fn certificate() -> Result<Certificate> {
    let directories = match ProjectDirs::from("", "", "wasm-server-runner") {
        Some(directories) => directories,
        None => {
            tracing::warn!("failed to determine application directory");
            return generate();
        }
    };

    let path = directories.data_local_dir();

    let certificate = match read(&path.join("certificate.der")) {
        Ok(Some(certificate)) => certificate,
        Ok(None) => return generate_in(path),
        Err(()) => return generate(),
    };

    let private_key = match read(&path.join("private_key.der")) {
        Ok(Some(private_key)) => private_key,
        Ok(None) => return generate_in(path),
        Err(()) => return generate(),
    };

    tracing::info!("re-using certificate from \"{}\"", path.display());

    Ok(Certificate { certificate, private_key })
}

fn read(path: &Path) -> Result<Option<Vec<u8>>, ()> {
    match std::fs::read(path) {
        Ok(file) => Ok(Some(file)),
        Err(error) => {
            if error.kind() == ErrorKind::NotFound {
                Ok(None)
            } else {
                tracing::error!("error reading file from \"{}\": {error}", path.display());
                Err(())
            }
        }
    }
}

fn write(path: &Path, data: &[u8]) -> Result<(), ()> {
    match std::fs::write(path, data) {
        Ok(()) => Ok(()),
        Err(error) => {
            tracing::error!("error saving file to \"{}\": {error}", path.display());
            Err(())
        }
    }
}

fn generate() -> Result<Certificate> {
    tracing::warn!("generated temporary certificate");

    generate_internal()
}

fn generate_in(path: &Path) -> Result<Certificate> {
    let certificate = generate_internal()?;

    if let Err(error) = std::fs::create_dir_all(path) {
        tracing::error!("error creating directory \"{}\": {error}", path.display());
        tracing::warn!("generated temporary certificate");
        return Ok(certificate);
    }

    if let Err(()) = write(&path.join("certificate.der"), &certificate.certificate)
        .and_then(|_| write(&path.join("private_key.der"), &certificate.private_key))
    {
        tracing::warn!("generated temporary certificate");
        return Ok(certificate);
    }

    tracing::info!("generated new certificate in \"{}\"", path.display());
    Ok(certificate)
}

fn generate_internal() -> Result<Certificate> {
    let certificate = rcgen::generate_simple_self_signed([String::from("localhost")])?;

    Ok(Certificate {
        certificate: certificate.serialize_der()?,
        private_key: certificate.serialize_private_key_der(),
    })
}
