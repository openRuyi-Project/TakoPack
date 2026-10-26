use std::{
    cell::OnceCell,
    path::{Path, PathBuf},
};

use cargo_metadata::{CargoOpt, Metadata};

use crate::{
    cargo::{cargo_dir::CargoDir, source::CargoSource},
    pkgdata::PkgdataBuilder,
};

/// A process for a source.
#[derive(Debug)]
pub struct Process<S>
where
    S: CargoSource,
{
    source: S,
    cargo_dir: OnceCell<CargoDir>,

    metadata: OnceCell<Metadata>,
    pkgdata: PkgdataBuilder,

    base_path: PathBuf,
    cargo_path: PathBuf,
    archive_path: PathBuf,
}

impl<S> Process<S>
where
    S: CargoSource,
{
    pub fn from(source: S, base_dir: &impl AsRef<Path>) -> Self {
        let id = source.id();

        let base_path: PathBuf = base_dir.as_ref().into();
        let cargo_path = base_path.join(&id);
        let archive_path = base_path.join(id.clone() + ".crate");

        Self {
            source,
            cargo_dir: OnceCell::new(),
            metadata: OnceCell::new(),
            pkgdata: PkgdataBuilder::new(),
            base_path,
            cargo_path,
            archive_path,
        }
    }

    pub fn load(&mut self) -> Result<&mut Self, Error>
    where
        S::Error: std::error::Error + Send + Sync + 'static,
    {
        let archive = self
            .source
            .load_archive(&self.base_path)
            .map_err(|e| Error::Io(std::io::Error::other(e)))?;
        self.pkgdata.checksum(archive.checksum.clone());

        let cargo_toml_path = archive.extract()?;
        let cargo_dir = CargoDir::new(
            cargo_toml_path
                .parent()
                .expect("SourceArchive::extract returns a manifest path"),
        );

        self.cargo_dir
            .set(cargo_dir)
            .map_err(|_| Error::DoubleInit)?;

        let metadata = self
            .cargo_dir
            .get()
            .ok_or(Error::GetBeforeInit)?
            .get_metadata(CargoOpt::AllFeatures)?;
        self.metadata.set(metadata).map_err(|_| Error::DoubleInit)?;

        Ok(self)
    }
}

#[derive(Debug)]
pub enum Error {
    DoubleInit,
    GetBeforeInit,
    CargoDir(crate::cargo::cargo_dir::Error),
    Io(std::io::Error),
}

impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

impl From<crate::cargo::cargo_dir::Error> for Error {
    fn from(value: crate::cargo::cargo_dir::Error) -> Self {
        Error::CargoDir(value)
    }
}
