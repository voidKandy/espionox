use crate::core::file_interface::{Directory, File, FileChunk};

pub trait Memorable {
    fn memorize(&self) -> String;
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

impl Memorable for File {
    fn memorize(&self) -> String {
        match self.summary.as_str() {
            "" => format!(
                "FilePath: {}, Content: {}",
                &self.filepath.display(),
                &self.content()
            ),
            _ => format!(
                "FilePath: {}, Content: {}, Summary: {}",
                &self.filepath.display(),
                &self.content(),
                &self.summary
            ),
        }
    }
}

impl Memorable for FileChunk {
    fn memorize(&self) -> String {
        format!(
            "FilePath: {}, ChunkIndex: {}, Content: {}",
            &self.parent_filepath.display(),
            &self.index,
            &self.content,
        )
    }
}
