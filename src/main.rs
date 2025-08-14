use clap::Parser;
use std::fs::File;
use std::path::Path;
use tar::Builder;

#[derive(Parser, Debug)]
#[clap(author = "Maxwell Rupp", version, about)]
/// Application configuration
struct Args {
    /// Print Verbose output
    #[arg(short = 'v', long = "verbose")]
    verbose: bool,

    /// Remove folders after tarballing
    #[arg(short = 'r', long = "remove")]
    remove: bool,

    /// Dry run - List folders to be tarballed but do not create tarballs
    #[arg(short = 'd', long = "dry-run")]
    dry_run: bool,

    /// Target folder - Tarball folders in this directory - Default is current directory
    #[arg()]
    target_dir: Option<String>,
}

fn main() {
    let args = Args::parse();

    let target_dir = target_dir_finder(args.target_dir);

    let tarball_names_and_paths = pathfinder(args.verbose, target_dir);

    tarballer(
        args.dry_run,
        args.verbose,
        args.remove,
        tarball_names_and_paths,
        target_dir,
    );
}

fn target_dir_finder(target_dir: Option<String>) -> &'static Path {
    match target_dir {
        Some(dir) => {
            let path = Path::new(&dir);
            if path.exists() {
                Box::leak(Box::new(path.to_path_buf())).as_path()
            } else {
                panic!("Target directory does not exist: {:?}", dir);
            }
        }
        None => {
            // If no target directory is provided, use the current directory
            Path::new(".")
        }
    }
}

/// Finds all folders in the current directory and returns a hashmap of tarball names and paths
fn pathfinder(
    verbose: bool,
    current_dir: &Path,
) -> std::collections::HashMap<String, std::path::PathBuf> {
    // find current directory
    if verbose {
        println!("Working directory: {:?}", current_dir);
    }

    // start vec of folder paths
    let mut folder_paths = Vec::new();

    // filter paths to only include folders
    let paths = std::fs::read_dir(current_dir).unwrap();
    for path in paths {
        let path = path.unwrap().path();
        if verbose {
            println!("Path: {:?}", path);
        }
        if path.is_dir() {
            if verbose {
                println!("Folder path detected: {:?}", path);
            }
            folder_paths.push(path);
        }
    }

    // start new hashmap for tarball names
    let mut tarball_names_and_paths = std::collections::HashMap::new();

    // iterate over folder paths and add to hashmap with {folderName}.tar as key and path as value
    for folder_path in folder_paths {
        let folder_name = folder_path.file_name().unwrap().to_str().unwrap();
        if verbose {
            println!("Folder name: {:?}", folder_name);
        }
        let tarball_name = format!("{}.tar", folder_name);
        if verbose {
            println!("Tarball name: {:?}", tarball_name);
        }
        tarball_names_and_paths.insert(tarball_name, folder_path);
    }

    // print hashmap if verbose
    if verbose {
        println!("Tarball names and paths: {:?}", tarball_names_and_paths);
    }

    tarball_names_and_paths
}

/// Creates tarballs from the folder paths in the hashmap
fn tarballer(
    dry_run: bool,
    verbose: bool,
    remove: bool,
    names_and_paths: std::collections::HashMap<String, std::path::PathBuf>,
    current_dir: &Path,
) {
    // iterate over hashmap and create tarballs
    for (tarball_name, folder_path) in names_and_paths {
        let tarball_name = tarball_name.to_string();
        if verbose {
            println!("Tarball name: {:?}", tarball_name);
        }
        let folder_path = folder_path.to_str().unwrap();
        if verbose {
            println!("Folder path: {:?}", folder_path);
        }
        let tarball_path = format!("{}/{}", current_dir.to_str().unwrap(), tarball_name);
        if verbose {
            println!("Tarball path: {:?}", tarball_path);
        }
        let tarball_path = tarball_path.to_string();
        if verbose {
            println!("Tarball path as String: {:?}", tarball_path);
        }
        match dry_run {
            true => {
                println!("Dry run - would tarball folder: {:?}", folder_path);
                match remove {
                    true => {
                        println!("Dry run - would remove folder: {:?}", folder_path);
                    }
                    false => {
                        println!("Dry run - would NOT remove folder: {:?}", folder_path);
                    }
                }
            }

            false => {
                if verbose {
                    println!("Tarballing folder: {:?}", folder_path);
                }
                let file = File::create(tarball_path).unwrap();
                let mut archive = Builder::new(file);
                archive.append_dir_all(folder_path, folder_path).unwrap();
                if verbose {
                    println!("Tarball created: {:?}", tarball_name);
                }
                match remove {
                    true => {
                        if verbose {
                            println!("Removing folder: {:?}", folder_path);
                        }
                        remove_dir(folder_path, verbose);
                    }
                    false => {
                        if verbose {
                            println!("Not removing folder: {:?}", folder_path);
                        }
                    }
                }
            }
        }
    }
}

fn remove_dir(path: &str, verbose: bool) {
    loop {
        if verbose {
            println!("Attempting to remove folder: {:?}", path);
        }
        let remover = std::fs::remove_dir_all(path);
        match remover {
            Ok(_) => {
                if verbose {
                    println!("Removed folder: {:?}", path);
                }
                break;
            }
            Err(e) => match e.kind() {
                std::io::ErrorKind::NotFound => {
                    if verbose {
                        println!("Folder not found: {:?}", path);
                    }
                    break;
                }
                std::io::ErrorKind::ResourceBusy => {
                    println!("Folder is busy: {:?}", path);
                    println!("Please close any open files in the folder and press Enter to retry.");
                    let mut input = String::new();
                    std::io::stdin().read_line(&mut input).unwrap();
                }
                std::io::ErrorKind::PermissionDenied => {
                    println!("Permission denied: {:?}", path);
                    println!(
                        "Please check your permissions (you may have a file open inside the directory) and press Enter to retry."
                    );
                    let mut input = String::new();
                    std::io::stdin().read_line(&mut input).unwrap();
                }
                _ => {
                    if verbose {
                        println!("Error removing folder: {:?}", e);
                    }
                    break;
                }
            },
        }
    }
}
