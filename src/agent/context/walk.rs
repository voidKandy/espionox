use crate::agent::{agents::SpecialAgent, handler::AgentHandler};
use std::error::Error;
use std::fmt;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct File {
    pub filepath: Box<Path>,
    pub content: String,
    pub content_embedding: Vec<f64>,
    pub summary: String,
    pub summary_embedding: Vec<f64>,
}

pub struct Directory {
    pub dirpath: Box<Path>,
    pub children: Vec<Directory>,
    pub files: Vec<File>,
}

impl fmt::Display for File {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.filepath.display().to_string())
    }
}

impl fmt::Display for Directory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let children_display: String = self
            .children
            .iter()
            .map(|child| child.dirpath.display().to_string())
            .collect::<Vec<String>>()
            .join("\n");

        let files_display: String = self
            .files
            .iter()
            .map(|file| file.filepath.display().to_string())
            .collect::<Vec<String>>()
            .join("\n");

        write!(
            f,
            "Name: {}\nChild Directories:\n{}\nFiles:\n{}\n",
            self.dirpath.display().to_string(),
            children_display,
            files_display
        )
    }
}

impl File {
    fn build(filepath: &str) -> File {
        File {
            content: fs::read_to_string(&filepath).unwrap_or_else(|e| e.to_string()),
            filepath: Path::new(filepath).into(),
            content_embedding: Vec::new(),
            summary: String::new(),
            summary_embedding: Vec::new(),
        }
    }
    pub async fn summarize(&mut self) -> Result<(), Box<dyn Error>> {
        let mut handler = AgentHandler::new(SpecialAgent::SummarizeAgent);
        handler
            .context
            .deliver_files_to_messages(vec![self.clone()], "Here is the file: ");
        match handler.prompt().await {
            Ok(summary) => Ok(self.summary = summary),
            Err(err) => Err(err),
        }
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
            // .max_depth(1)
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
