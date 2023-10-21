use core::fmt;

use crate::{
    core::{Directory, File, Io},
    features::long_term_memory::{
        database::SqlFromFlattenableStruct, models::file::CreateFileBody,
    },
    memory::MessageVector,
};

use super::ToMessage;

#[derive(Clone, Debug)]
pub struct MemoryCache {
    pub messages: MessageVector,
    pub cached_structs: Option<Vec<FlattenedStruct>>,
}

#[derive(Clone, Debug)]
pub enum FlattenedStruct {
    File(BuildFrom),
    Directory(BuildFrom),
    // Io,
}

#[derive(Clone, Debug)]
pub enum BuildFrom {
    String(String),
}

impl fmt::Display for FlattenedStruct {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self,)
    }
}

impl ToMessage for FlattenedStruct {
    fn with_default_role(&self) -> super::Message {
        match self {
            FlattenedStruct::File(_) => File::rebuild(self.to_owned())
                .expect("Failed to rebuild file")
                .with_default_role(),
            FlattenedStruct::Directory(_) => Directory::rebuild(self.to_owned())
                .expect("Failed to rebuild file")
                .with_default_role(),
        }
    }
    // This should never be called but needs to be defined for compilation
    fn role() -> super::MessageRole {
        super::MessageRole::System
    }
}

impl From<String> for BuildFrom {
    fn from(value: String) -> Self {
        Self::String(value)
    }
}

impl TryInto<String> for BuildFrom {
    type Error = anyhow::Error;
    fn try_into(self) -> Result<String, Self::Error> {
        match self {
            Self::String(str) => Ok(str),
            _ => Err(anyhow::anyhow!("Build from is not of string variant")),
        }
    }
}

pub trait FlattenStruct: std::fmt::Debug + ToMessage {
    fn flatten(self) -> FlattenedStruct;
    fn rebuild(flattened: FlattenedStruct) -> Result<Self, anyhow::Error>
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
    fn flatten(self) -> FlattenedStruct {
        let build_from = self.filepath.to_string_lossy().to_string().into();
        FlattenedStruct::File(build_from)
    }
    fn rebuild(flattened: FlattenedStruct) -> Result<Self, anyhow::Error>
    where
        Self: Sized,
    {
        match flattened {
            FlattenedStruct::File(build_from) => {
                let path: String = build_from.try_into().unwrap();
                Ok(File::from(path.as_str()))
            }
            _ => Err(anyhow::anyhow!("Flattened string is not a file path")),
        }
    }
}

impl FlattenStruct for Directory {
    fn flatten(self) -> FlattenedStruct {
        let build_from = self.dirpath.to_string_lossy().to_string().into();
        FlattenedStruct::Directory(build_from)
    }
    fn rebuild(flattened: FlattenedStruct) -> Result<Self, anyhow::Error>
    where
        Self: Sized,
    {
        match flattened {
            FlattenedStruct::Directory(build_from) => {
                let path: String = build_from.try_into().unwrap();
                Ok(Directory::from(path.as_str()))
            }
            _ => Err(anyhow::anyhow!("Flattened string is not a dir path")),
        }
    }
}

// impl FlattenStruct for Io {
//     fn flatten(self) -> FlattenedStruct {
//         let build_from = self.i.into();
//         FlattenedStruct {
//             build_from,
//             flat_type: FlatType::Io,
//         }
//     }
//     fn rebuild(flattened: FlattenedStruct) -> Result<Self, anyhow::Error>
//     where
//         Self: Sized,
//     {
//         match flattened.flat_type {
//             FlatType::Io => {
//                 let input: &str = flattened.build_from.try_into().unwrap();
//                 let mut io = Io::init(input);
//                 io.run_input();
//                 Ok(io)
//             }
//             _ => Err(anyhow::anyhow!("Flattened struct is file or directory")),
//         }
//     }
// }

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

impl MemoryCache {
    fn push_flattenable_struct(&mut self, flat: impl FlattenStruct) {
        match &mut self.cached_structs {
            Some(structs) => {
                structs.push(flat.flatten());
            }
            None => self.cached_structs = Some(vec![flat.flatten()]),
        }
    }
}
