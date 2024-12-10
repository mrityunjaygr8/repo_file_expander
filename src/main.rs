use git2::Repository;
use std::fs::File;
use std::io::{self, Read};
use std::path::{Path, PathBuf};
use url::Url;

/// Represents different types of input sources
#[derive(Debug, PartialEq)]
enum SourceType {
    LocalDirectory,
    GitRepository,
    Unknown,
}

/// Handles reading content from different source types
struct SourceContentReader {
    path: String,
    location: Option<PathBuf>,
    source_type: SourceType,
    temp_dir: Option<TempDir>,
}

impl SourceContentReader {
    /// Create a new SourceContentReader
    fn new(path: &str) -> Self {
        SourceContentReader {
            path: path.to_string(),
        }
    }

    /// Identify the type of source
    fn identify_source(&self) -> SourceType {
        // First, check if it's a local directory
        if self.is_local_directory() {
            return SourceType::LocalDirectory;
        }

        // Then check if it's a git repository URL
        if self.is_git_repository() {
            return SourceType::GitRepository;
        }

        // If neither, return Unknown
        SourceType::Unknown
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
    fn read_file_contents(&self, filename: &str) -> Result<String, Box<dyn std::error::Error>> {
        match self.identify_source() {
            SourceType::LocalDirectory => {
                // Read from local directory
                Ok(self.read_local_file(filename)?)
            }
            SourceType::GitRepository => {
                // Attempt to read from local git repository or clone remote repository
                self.read_git_repository_file(filename)
            }
            SourceType::Unknown => Err("Unable to identify source type".into()),
        }
    }

    /// Read file from local directory
    fn read_local_file(&self, filename: &str) -> Result<String, io::Error> {
        let file_path = Path::new(&self.path).join(filename);

        // Open the file
        let mut file = File::open(file_path)?;

        // Read contents to string
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;

        Ok(contents)
    }

    /// Read file from git repository (local or remote)
    fn read_git_repository_file(
        &self,
        filename: &str,
    ) -> Result<String, Box<dyn std::error::Error>> {
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

        // Construct full file path
        let file_path = repo_path.join(filename);

        // Open and read the file
        let mut file = File::open(file_path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;

        Ok(contents)
    }

    // /// Check if a specific file exists
    // fn file_exists(&self, filename: &str) -> bool {
    //     match self.identify_source() {
    //         SourceType::LocalDirectory => Path::new(&self.path).join(filename).exists(),
    //         SourceType::GitRepository => {
    //             println!("=======  here  =============");
    //             // For git repository, check if file exists in the repository
    //             let repo_path = if self.is_local_directory() {
    //                 // If it's a local git repository, use the existing path
    //                 PathBuf::from(&self.path)
    //             } else {
    //                 // Clone remote repository to a temporary directory
    //                 let temp_dir = tempfile::tempdir()?;
    //                 let repo_path = temp_dir.path().to_path_buf();
    //
    //                 // Clone the repository
    //                 Repository::clone(&self.path, &repo_path)?;
    //
    //                 repo_path
    //             };
    //
    //             repo_path.join(filename).exists()
    //         }
    //         SourceType::Unknown => false,
    //     }
    // }
}

/// Demonstrate the usage of SourceContentReader
fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Test cases for different source types
    let test_sources = vec![
        "/home/mgr8/stuff/rust/my-redis/",        // Local git repository
        "https://github.com/mrityunjaygr8/guzei", // Remote git repository
        "../../oss/devenv/",                      // Local git repository
        "invalid/path",                           // Unknown source
    ];

    // File to search for in each source
    let target_file = "devenv.nix";

    for source in test_sources {
        println!("\nAnalyzing source: {}", source);

        let reader = SourceContentReader::new(source);

        // Identify source type
        let source_type = reader.identify_source();
        println!("Source Type: {:?}", source_type);

        // Try to read file contents
        match reader.read_file_contents(target_file) {
            Ok(contents) => {
                println!("File contents (first 200 chars):");
                println!("{}", &contents[..contents.len().min(200)]);
            }
            Err(e) => println!("Error reading file: {}", e),
        }
    }

    Ok(())
}
