use git2::Repository;
use include_dir::{include_dir, Dir};
use std::fs::File;
use std::io::{self, Read};
use std::path::{Path, PathBuf};
use std::str::FromStr;
use tempfile::TempDir;
use url::Url;

static PROJECT_DIR: Dir = include_dir!("$CARGO_MANIFEST_DIR/init");

/// Represents different types of input sources
#[derive(Debug, PartialEq)]
enum SourceType {
    LocalDirectory,
    GitRepository,
    Unknown,
}

/// Handles reading content from different source types
pub struct SourceContentReader {
    path: String,
    location: Option<PathBuf>,
    source_type: SourceType,
    temp_dir: Option<TempDir>,
}

impl SourceContentReader {
    /// Create a new SourceContentReader
    pub fn new(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let mut scr = SourceContentReader {
            path: path.to_string(),
            location: None,
            source_type: SourceType::Unknown,
            temp_dir: None,
        };
        if scr.is_local_directory() {
            scr.location = PathBuf::from_str(&scr.path).ok();
            scr.source_type = SourceType::LocalDirectory;
        }

        // Then check if it's a git repository URL
        if scr.is_git_repository() {
            scr.setup_git_repository()?;
            scr.source_type = SourceType::GitRepository;
        }

        Ok(scr)
    }

    /// Check if the path is a local directory
    fn is_local_directory(&self) -> bool {
        let path = Path::new(&self.path);
        path.exists() && path.is_dir()
    }

    /// Check if the path is a git repository URL or local git repository
    fn is_git_repository(&self) -> bool {
        // Check if it's a remote git URL
        if let Ok(url) = Url::parse(&self.path) {
            return self.validate_git_url(&url);
        }

        // Check if it's a local git repository
        let path = Path::new(&self.path);
        path.exists() && path.join(".git").exists()
    }

    /// Validate git repository URL
    fn validate_git_url(&self, url: &Url) -> bool {
        let git_hosts = ["github.com", "gitlab.com", "bitbucket.org"];

        url.scheme() == "https"
            && git_hosts.contains(&url.host_str().unwrap_or(""))
            && (url.path().ends_with(".git") || url.path().contains("/"))
    }

    /// Read contents of a specific file based on source type
    pub fn read_file_contents(&self, filename: &str) -> Result<String, Box<dyn std::error::Error>> {
        match self.source_type {
            SourceType::LocalDirectory | SourceType::GitRepository => {
                // Read from local directory
                Ok(self.read_local_file(filename)?)
            }
            SourceType::Unknown => Ok(self.read_fallback(filename)?),
        }
    }

    fn read_fallback(&self, filename: &str) -> Result<String, io::Error> {
        let file_path = PROJECT_DIR.get_file(filename).unwrap();
        let body = file_path.contents_utf8().unwrap();
        return Ok(body.to_string());
    }

    /// Read file from local directory
    fn read_local_file(&self, filename: &str) -> Result<String, io::Error> {
        let file_path = self.location.as_ref().unwrap().join(filename);

        if !file_path.exists() {
            return self.read_fallback(filename);
        }

        // Open the file
        let mut file = File::open(file_path)?;

        // Read contents to string
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;

        Ok(contents)
    }

    fn setup_git_repository(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let temp_dir = tempfile::tempdir()?;
        // Temporary directory for cloning if it's a remote repository
        let repo_path = if self.is_local_directory() {
            // If it's a local git repository, use the existing path
            PathBuf::from(&self.path)
        } else {
            // Clone remote repository to a temporary directory
            let repo_path = temp_dir.path().to_path_buf();

            // Clone the repository
            Repository::clone(&self.path, &repo_path)?;
            repo_path
        };
        self.location = Some(repo_path);
        self.temp_dir = Some(temp_dir);
        Ok(())
    }
}
