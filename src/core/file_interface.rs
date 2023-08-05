use std::fs;
use std::path::Path;
use tracing::{self, info};

#[derive(Debug, Clone)]
pub struct File {
    pub filepath: Box<Path>,
    pub chunks: Vec<FileChunk>,
    pub summary: String,
    pub summary_embedding: Vec<f32>,
}

#[derive(Debug, Clone)]
pub struct FileChunk {
    pub parent_filepath: Box<Path>,
    pub content: String,
    pub content_embedding: Vec<f32>,
    pub index: i16,
}

#[derive(Debug, Clone)]
pub struct Directory {
    pub dirpath: Box<Path>,
    pub children: Vec<Directory>,
    pub files: Vec<File>,
}

impl File {
    pub fn build(filename: &str) -> File {
        let filepath = fs::canonicalize(Path::new(filename)).unwrap().into();
        File {
            filepath,
            chunks: vec![],
            summary: String::new(),
            summary_embedding: Vec::new(),
        }
        .chunkify()
    }

    #[tracing::instrument]
    pub fn chunkify(&mut self) -> Self {
        info!("chunkifying: {}", &self.filepath.to_str().unwrap());
        let content = fs::read_to_string(&self.filepath).unwrap_or_else(|e| {
            println!(
                "Failed to get content of {}\nError: {e:?}",
                &self.filepath.to_str().unwrap()
            );
            e.to_string()
        });
        let lines: Vec<&str> = content.lines().collect();
        lines.chunks(50).enumerate().for_each(|(i, c)| {
            self.chunks.push(FileChunk {
                parent_filepath: self.filepath.clone(),
                content: c.join("\n"),
                content_embedding: Vec::new(),
                index: i as i16,
            });
        });
        self.to_owned()
    }

    pub fn content(&self) -> String {
        let mut content: Vec<&str> = Vec::new();
        self.chunks.iter().for_each(|c| content.push(&c.content));
        content.join("\n")
    }
}

impl Directory {
    pub fn build(path: &str) -> Result<Directory, Box<dyn std::error::Error>> {
        let dirpath = Path::new(path);
        let (children, files) =
            Directory::walk_directory(dirpath).expect("Failure walking directory");
        Ok(Directory {
            dirpath: dirpath.into(),
            children,
            files,
        })
    }

    fn walk_directory(
        root: &Path,
    ) -> Result<(Vec<Directory>, Vec<File>), Box<dyn std::error::Error>> {
        let directory_iterator = fs::read_dir(root)
            .expect("Couldn't read root dir")
            .into_iter()
            .filter_map(|entry| entry.ok().map(|path| path.path()));

        let (mut children, mut files) = (vec![], vec![]);
        for entry in directory_iterator {
            match &entry.is_dir() {
                true => {
                    children.push(Directory::build(entry.to_str().unwrap()).unwrap());
                }
                false => {
                    files.push(File::build(&entry.display().to_string()));
                }
            }
        }
        Ok((children, files))
    }
}
