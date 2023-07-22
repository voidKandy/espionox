use std::fmt;
use std::fs;
use std::path::Path;
use std::process::Command;
use std::process::Stdio;

#[derive(Debug, Clone)]
pub struct File {
    pub filepath: Box<Path>,
    pub chunks: Vec<FileChunk>,
    pub summary: String,
    pub summary_embedding: Vec<f64>,
}

#[derive(Debug, Clone)]
pub struct FileChunk {
    pub parent_filepath: Box<Path>,
    pub content: String,
    pub content_embedding: Vec<f64>,
    pub index: u32,
}

#[derive(Debug, Clone)]
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
    pub fn build(filepath: &str) -> File {
        File {
            filepath: Path::new(&Self::full_filepath(filepath)).into(),
            chunks: vec![],
            summary: String::new(),
            summary_embedding: Vec::new(),
        }
        .chunkify()
    }

    fn full_filepath(filename: &str) -> String {
        let args: &[&str] = &["readlink", "-f", filename];
        let out = Command::new(args[0])
            .args(&args[1..])
            .stdout(Stdio::piped())
            .output()
            .expect("failed to execute tmux command");
        print!(
            "{}",
            String::from_utf8_lossy(&out.stdout.clone()).to_string()
        );
        String::from_utf8_lossy(&out.stdout).to_string()
    }

    pub fn chunkify(&mut self) -> Self {
        let content = fs::read_to_string(&self.filepath).unwrap_or_else(|e| e.to_string());
        let lines: Vec<&str> = content.lines().collect();
        lines.chunks(50).enumerate().for_each(|(i, c)| {
            self.chunks.push(FileChunk {
                parent_filepath: self.filepath.clone(),
                content: c.join("\n"),
                content_embedding: Vec::new(),
                index: i as u32,
            });
        });
        self.to_owned()
        // chunks.iter().for_each(|c| println!("{}", c.content));
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
