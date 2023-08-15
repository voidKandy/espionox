use super::File;
use super::Memorable;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct Directory {
    pub dirpath: Box<Path>,
    pub children: Vec<Directory>,
    pub files: Vec<File>,
}

impl Directory {
    pub fn build(path: &str) -> Directory {
        let dirpath = fs::canonicalize(Path::new(path)).unwrap();
        let (children, files) =
            Directory::get_children_and_files(&dirpath).expect("Failure walking directory");
        Directory {
            dirpath: dirpath.into(),
            children,
            files,
        }
    }

    fn get_children_and_files(
        root: &Path,
    ) -> Result<(Vec<Directory>, Vec<File>), Box<dyn std::error::Error>> {
        let directory_iterator = fs::read_dir(root)
            .expect("Couldn't read root dir")
            .into_iter()
            .filter_map(|entry| entry.ok().map(|path| path.path()));

        let excluded_paths = Self::generate_excluded_paths(root);
        let (mut children, mut files) = (vec![], vec![]);
        for entry in directory_iterator {
            if excluded_paths
                .iter()
                .map(|p| Path::new(p))
                .any(|p| fs::canonicalize(p).unwrap() == entry.as_path())
            {
                continue;
            }
            match &entry.is_dir() {
                true => {
                    children.push(Directory::build(entry.to_str().unwrap()));
                }
                false => {
                    files.push(File::build(&entry.display().to_string()));
                }
            }
        }
        Ok((children, files))
    }

    fn generate_excluded_paths(dirpath: &Path) -> Vec<String> {
        let mut excluded_paths = vec![String::from(".git")];

        if dirpath.join("package.json").is_file() {
            excluded_paths.push(String::from("node_modules"));
        }

        if dirpath.join("Cargo.toml").is_file() {
            excluded_paths.push(String::from("target"));
            excluded_paths.push(String::from("Cargo.lock"));
        }

        excluded_paths
    }
}

impl Memorable for Directory {
    fn memorize(&self) -> String {
        let mut files_payload = vec![];
        self.files.iter().for_each(|f| {
            files_payload.push(match f.summary.as_str() {
                "" => format!(
                    "FilePath: {}, Content: {}",
                    &f.filepath.display(),
                    &f.content()
                ),
                _ => format!(
                    "FilePath: {}, Content: {}, Summary: {}",
                    &f.filepath.display(),
                    &f.content(),
                    &f.summary
                ),
            })
        });
        format!(
            "Relevant Directory path: {}, Child Directories: [{:?}], Files: [{}]",
            self.dirpath.display().to_string(),
            self.children
                .clone()
                .into_iter()
                .map(|c| c.dirpath.display().to_string())
                .collect::<Vec<String>>()
                .join(", "),
            files_payload.join(", ")
        )
    }
}
