use regex::Regex;

#[derive(Clone, Debug)]
pub enum Path {
    Regex(Regex),
    Exact(String),
}

impl From<Regex> for Path {
    fn from(path: Regex) -> Path {
        Path::Regex(path)
    }
}

impl From<String> for Path {
    fn from(path: String) -> Path {
        Path::Exact(path)
    }
}

impl From<&'static str> for Path {
    fn from(path: &str) -> Path {
        Path::Exact(path.to_string())
    }
}
