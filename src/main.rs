use clap::crate_version;
use cli::Commands;

mod cli;
mod stuff;

/// Demonstrate the usage of SourceContentReader
fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Test cases for different source types
    // let test_sources = vec![
    //     "/home/mgr8/stuff/rust/my-redis/",        // Local git repository
    //     "https://github.com/mrityunjaygr8/guzei", // Remote git repository
    //     "../../golang/guzei",                     // Local git repository
    //     "invalid/path",                           // Unknown source
    // ];

    let cli = cli::Cli::parse_and_resolve_options();

    let print_version = || {
        println!("rfe {} ", crate_version!());
        Ok(())
    };

    let command = match cli.command {
        None => return print_version(),
        Some(cmd) => cmd,
    };

    match command {
        Commands::Init { target, source } => {
            println!("target: {:?}, source: {:?}", target, source);
            let target_file = "devenv.nix";

            let reader = stuff::SourceContentReader::new(source.unwrap().as_str()).unwrap();

            // Try to read file contents
            match reader.read_file_contents(target_file) {
                Ok(contents) => {
                    println!("File contents (first 200 chars):");
                    println!("{}", &contents[..contents.len().min(200)]);
                }
                Err(e) => println!("Error reading file: {}", e),
            }
        }
    }

    // // File to search for in each source

    Ok(())
}
