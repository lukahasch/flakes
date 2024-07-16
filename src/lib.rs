use std::path::Path;
use std::{io, path::PathBuf};
use std::fs::File;
use std::io::Read;
use serde_derive::{Deserialize, Serialize};
use utils::hash;

pub mod utils;

pub trait Builder where Self: Sized {
    fn build(source: impl Read, input: &serde_json::Value, store: &mut Store<Single, Self>) -> io::Result<serde_json::Value>; 
}

#[derive(Serialize, Deserialize, PartialEq, Clone)]
pub struct Store<State, B: Builder> {
    pub path: PathBuf,
    pub state: State,
    _b: std::marker::PhantomData<B>,
}

pub fn store<B: Builder>() -> Store<General, B> {
    Store::<General,B>::new(std::env::var("FLAKES_STORE").unwrap_or("./flakes".into()).into())
}

#[derive(Serialize, Deserialize, PartialEq, Clone)]
pub struct General;

impl<S, B: Builder> Store<S, B> {
    pub fn build_path(&self, name: &str, input: &serde_json::Value) -> PathBuf {
        self.path.join(name).join(hash(input).to_string())
    }

    pub fn check_dependencies(&self, path: &PathBuf) -> io::Result<bool> {
        let dependencies: Vec<Dependency> = serde_json::from_reader(File::open(path.join(".deps"))?)?;
        for i in dependencies {
            if !i.is_consisten()? {
                return Ok(false);
            }
        }
        Ok(true)
    }

    pub fn get(&self, name: &str, input: &serde_json::Value) -> io::Result<Option<(PathBuf, serde_json::Value)>> {
        let path = self.build_path(name, input);
        if path.exists() {
            if !self.check_dependencies(&path)? {
                return Ok(None);
            }
            Ok(Some((path.clone(), serde_json::from_reader(File::open(path.join(".output"))?)?)))
        } else {
            Ok(None)
        }
    }

    pub fn remove_create(&self, path: &PathBuf) -> io::Result<()> {
        if path.exists() {
            std::fs::remove_dir_all(path)?;
        }
        std::fs::create_dir_all(path)?;
        Ok(())
    }

    pub fn create(&self, name: &str, source: impl Read + Clone, input: &serde_json::Value) -> io::Result<(PathBuf, serde_json::Value)> {
        if let Ok(Some((path, result))) = self.get(name, input) {
            return Ok((path, result));
        }
        let path = self.build_path(name, input);
        self.remove_create(&path)?;
        let mut store = Store {
            path: self.path.clone(),
            state: Single { path: path.clone(), dependencies: Vec::new() },
            _b: std::marker::PhantomData,
        };
        let result = B::build(source, input, &mut store)?;
        serde_json::to_writer(File::create(path.join(".output"))?, &result)?;
        store.finalize()?;
        Ok((std::fs::canonicalize(path)?, result))
    }
}

impl<B: Builder> Store<General, B> {
    pub fn new(path: PathBuf) -> Self {
        Self {
            path,
            state: General,
            _b: std::marker::PhantomData,
        }
    }
}

#[derive(Serialize, Deserialize, PartialEq, Clone)]
pub struct Single {
   pub path: PathBuf,
   pub dependencies: Vec<Dependency>,
}

impl<B: Builder> Store<Single, B> {
    pub fn is_outside(&self, path: &PathBuf) -> bool {
        !path.starts_with(&self.state.path)
    }

    pub fn add_dep(&mut self, dependency: Dependency) {
        self.state.dependencies.push(dependency);
    }

    pub fn create_file(&self, path: impl AsRef<Path>) -> io::Result<File> {
        Ok(File::create(self.state.path.join(path.as_ref()))?)
    }

    pub fn open_file(&mut self, path: impl AsRef<Path>) -> io::Result<File> {
        if self.is_outside(&path.as_ref().to_path_buf()) {
            self.add_dep(Dependency::file(path.as_ref().to_path_buf())?);
        }
        Ok(File::open(path)?)
    }

    pub fn create_dir(&self, path: impl AsRef<Path>) -> io::Result<()> {
        Ok(std::fs::create_dir_all(self.state.path.join(path.as_ref()))?)
    }

    pub fn finalize(self) -> io::Result<()> {
        serde_json::to_writer(File::create(self.state.path.join(".deps"))?, &self.state.dependencies)?;
        Ok(())
    }
}

#[derive(Serialize, Deserialize, PartialEq, Clone)]
pub enum Dependency {
    File {
        pathbuf: PathBuf,
        checksum: u64,
    }
}

impl Dependency {
    pub fn is_consisten(&self) -> io::Result<bool> {
        Ok(match self {
            Dependency::File { pathbuf, checksum } => {
                if pathbuf.exists() {
                    let buf = std::fs::read(pathbuf)?;
                    hash(&buf) == *checksum
                } else {
                    false
                }
            }
        })
    }

    pub fn file(path: PathBuf) -> io::Result<Self> {
        let path = std::fs::canonicalize(path)?;
        let buf = std::fs::read(&path)?;
        Ok(Dependency::File { pathbuf: path, checksum: hash(&buf) })
    }
}