use std::path::PathBuf;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ImageMark {
    pub n: usize,
    pub format: String,
    pub width: u32,
    pub height: u32,
    pub bytes: u64,
    pub path: PathBuf,
}

impl ImageMark {
    pub fn token(&self) -> String {
        format!("[Image #{}]", self.n)
    }
}
