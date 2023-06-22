use std::fmt;
use std::fs;
use std::path::Path;
use walkdir::WalkDir;

pub struct Directory {
    pub name: String,
    pub children: Vec<Directory>,
    pub files: Vec<File>,
}

impl fmt::Display for Directory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let children_display: String = self
            .children
            .iter()
            .map(|child| child.name.clone())
            .collect::<Vec<String>>()
            .join("\n");

        let files_display: String = self
            .files
            .iter()
            .map(|file| file.name.clone())
            .collect::<Vec<String>>()
            .join("\n");

        write!(
            f,
            "Name: {}\nChild Directories:\n{}\nFiles:\n{}\n",
            self.name, children_display, files_display
        )
    }
}

#[derive(Debug)]
pub struct File {
    pub name: String,
    pub content: String,
    pub content_embedding: Vec<f64>,
    pub summary: String,
    pub summary_embedding: Vec<f64>,
}

impl fmt::Display for File {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}
impl File {
    fn new() -> File {
        File {
            name: String::new(),
            content: String::new(),
            content_embedding: Vec::new(),
            summary: String::new(),
            summary_embedding: Vec::new(),
        }
    }
}

// Returns parent directory as an object
pub fn walk_directory(root: &str) -> Result<Directory, Box<dyn std::error::Error>> {
    let directory_iterator = WalkDir::new(root)
        .max_depth(1)
        .into_iter()
        .filter_map(|entry| {
            entry
                .ok()
                .map(|dir_entry| dir_entry.path().to_owned())
                .filter(|path| path.display().to_string() != root)
        });

    let mut directory = Directory {
        name: root.to_string(),
        children: Vec::new(),
        files: Vec::new(),
    };
    for entry in directory_iterator {
        match &entry.is_dir() {
            true => {
                directory
                    .children
                    .push(walk_directory(&entry.display().to_string()).unwrap());
            }
            false => {
                let mut newfile = File::new();
                match read_file(&entry) {
                    Ok((name, content)) => {
                        newfile.name = name;
                        newfile.content = content;
                        directory.files.push(newfile);
                    }
                    Err(e) => println!("{:?}", e),
                }
            }
        }
    }
    Ok(directory)
}

// Should return content & name of file
fn read_file(file: &Path) -> Result<(String, String), &'static str> {
    let name = file.display().to_string();
    match fs::read_to_string(&file) {
        Ok(content) => Ok((name, content)),
        Err(err) => {
            println!("Problem opening file: {err:?}");
            Err("catch all")
        }
    }
}
