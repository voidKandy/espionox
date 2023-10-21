use crate::{
    core::{Directory, File, FileChunk, Io},
    memory::MessageVector,
};
use std::path::PathBuf;

#[derive(Clone, Debug)]
pub struct MemoryCache {
    pub messages: MessageVector,
    pub cached_structs: Option<Vec<FlattenedCachedStruct>>,
}

#[derive(Clone, Debug)]
pub struct FlattenedCachedStruct {
    string: String,
    metadata: Option<CachedStructMetadata>,
}

#[derive(Clone, Debug)]
struct CachedStructMetadata {
    index: Option<i16>,
}

pub trait FlattenStruct: std::fmt::Debug {
    fn flatten(self) -> FlattenedCachedStruct;
    fn rebuild(flattened: FlattenedCachedStruct) -> Result<Self, anyhow::Error>
    where
        Self: Sized;
}

impl From<MessageVector> for MemoryCache {
    fn from(messages: MessageVector) -> Self {
        Self {
            messages,
            cached_structs: None,
        }
    }
}

impl FlattenStruct for File {
    fn flatten(self) -> FlattenedCachedStruct {
        let string = self.filepath.to_string_lossy().to_string();
        FlattenedCachedStruct {
            string,
            metadata: None,
        }
    }
    fn rebuild(flattened: FlattenedCachedStruct) -> Result<Self, anyhow::Error>
    where
        Self: Sized,
    {
        if flattened.is_file() {
            Ok(File::from(flattened.string.as_str()))
        } else {
            Err(anyhow::anyhow!("Flattened string is not a file path"))
        }
    }
}

impl FlattenStruct for FileChunk {
    fn flatten(self) -> FlattenedCachedStruct {
        let string = self.parent_filepath.to_string_lossy().to_string();
        let metadata = Some(CachedStructMetadata {
            index: Some(self.index),
        });
        FlattenedCachedStruct { string, metadata }
    }
    fn rebuild(flattened: FlattenedCachedStruct) -> Result<Self, anyhow::Error>
    where
        Self: Sized,
    {
        if flattened.is_file() {
            if let Some(metadata) = flattened.metadata {
                match metadata.index {
                    Some(idx) => {
                        let path = PathBuf::from(flattened.string);
                        let file = File::from(path);
                        Ok(file.chunks.get(idx as usize).unwrap().to_owned())
                    }
                    None => Err(anyhow::anyhow!("No index is in metadata")),
                }
            } else {
                Err(anyhow::anyhow!("No metadata"))
            }
        } else {
            Err(anyhow::anyhow!("Flattened struct is not a file chunk"))
        }
    }
}

impl FlattenStruct for Directory {
    fn flatten(self) -> FlattenedCachedStruct {
        let string = self.dirpath.to_string_lossy().to_string();
        FlattenedCachedStruct {
            string,
            metadata: None,
        }
    }
    fn rebuild(flattened: FlattenedCachedStruct) -> Result<Self, anyhow::Error>
    where
        Self: Sized,
    {
        if flattened.is_dir() {
            Ok(Directory::from(flattened.string.as_str()))
        } else {
            Err(anyhow::anyhow!("Flattened string is not a dir path"))
        }
    }
}

impl FlattenStruct for Io {
    fn flatten(self) -> FlattenedCachedStruct {
        let string = self.i;
        FlattenedCachedStruct {
            string,
            metadata: None,
        }
    }
    fn rebuild(flattened: FlattenedCachedStruct) -> Result<Self, anyhow::Error>
    where
        Self: Sized,
    {
        if !flattened.is_file() && !flattened.is_dir() {
            let mut io = Io::init(&flattened.string);
            io.run_input();
            Ok(io)
        } else {
            Err(anyhow::anyhow!("Flattened struct is file or directory"))
        }
    }
}

impl FlattenedCachedStruct {
    fn is_file(&self) -> bool {
        PathBuf::from(self.string.to_string()).is_file()
    }
    fn is_dir(&self) -> bool {
        PathBuf::from(self.string.to_string()).is_dir()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum CachingMechanism {
    Forgetful,
    SummarizeAtLimit { limit: usize, save_to_lt: bool },
}

impl Default for CachingMechanism {
    fn default() -> Self {
        Self::default_summary_at_limit()
    }
}

impl CachingMechanism {
    pub fn limit(&self) -> usize {
        match self {
            Self::Forgetful => 2, // Only allows for user in agent out
            Self::SummarizeAtLimit { limit, .. } => *limit as usize,
        }
    }
    pub fn long_term_enabled(&self) -> bool {
        match self {
            Self::Forgetful => false,
            Self::SummarizeAtLimit { save_to_lt, .. } => *save_to_lt,
        }
    }
    pub fn default_summary_at_limit() -> Self {
        let limit = 50;
        let save_to_lt = false;
        CachingMechanism::SummarizeAtLimit { limit, save_to_lt }
    }
}
