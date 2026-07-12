use std::{
    fs::{self, File},
    io::{self, Write},
    path::{Path, PathBuf},
};

use base64::{Engine, engine::general_purpose::STANDARD};
use clap::{Parser, Subcommand};
use ed25519_dalek::{SigningKey, VerifyingKey};
use rand::rngs::OsRng;
use rayslash_module_manifest::{
    MAX_ICON_BYTES, MAX_PACKAGE_BYTES, MAX_WASM_BYTES, ModuleKind, ModuleManifest,
};
use sha2::{Digest, Sha256};
use walkdir::WalkDir;

#[derive(Parser)]
#[command(
    name = "rayslash-module",
    version,
    about = "Validate and package rayslash modules"
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    Validate {
        #[arg(default_value = ".")]
        directory: PathBuf,
        #[arg(long)]
        official: bool,
    },
    Package {
        #[arg(default_value = ".")]
        directory: PathBuf,
        #[arg(long)]
        output: Option<PathBuf>,
        #[arg(long)]
        official: bool,
    },
    Keygen {
        #[arg(long)]
        key_id: String,
        #[arg(long)]
        output: PathBuf,
    },
}

fn main() {
    if let Err(error) = run() {
        eprintln!("error: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    match Cli::parse().command {
        Command::Validate {
            directory,
            official,
        } => {
            let manifest = validate_directory(&directory, official)?;
            println!(
                "valid {} {} ({})",
                manifest.id, manifest.version, manifest.kind
            );
        }
        Command::Package {
            directory,
            output,
            official,
        } => package(&directory, output, official)?,
        Command::Keygen { key_id, output } => keygen(&key_id, &output)?,
    }
    Ok(())
}

fn validate_directory(
    directory: &Path,
    official: bool,
) -> Result<ModuleManifest, Box<dyn std::error::Error>> {
    let manifest = ModuleManifest::load(&directory.join("module.toml"), official)?;
    require_file(directory, "README.md", None)?;
    require_file(directory, "LICENSE", None)?;
    require_file(
        directory,
        manifest.icon.to_string_lossy().as_ref(),
        Some(MAX_ICON_BYTES),
    )?;
    if manifest.kind == ModuleKind::Wasm {
        require_file(directory, "module.wasm", Some(MAX_WASM_BYTES))?;
    }
    Ok(manifest)
}

fn require_file(
    directory: &Path,
    relative: &str,
    limit: Option<u64>,
) -> Result<(), Box<dyn std::error::Error>> {
    let path = directory.join(relative);
    let metadata = fs::symlink_metadata(&path)?;
    if !metadata.file_type().is_file() || metadata.file_type().is_symlink() {
        return Err(format!("{} must be a regular package file", path.display()).into());
    }
    if limit.is_some_and(|limit| metadata.len() > limit) {
        return Err(format!("{} exceeds its size limit", path.display()).into());
    }
    Ok(())
}

fn package(
    directory: &Path,
    output: Option<PathBuf>,
    official: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let manifest = validate_directory(directory, official)?;
    let output = output
        .unwrap_or_else(|| PathBuf::from(format!("{}-{}.tar.zst", manifest.id, manifest.version)));
    let temporary = output.with_extension("tar.zst.tmp");
    let file = File::create(&temporary)?;
    let encoder = zstd::Encoder::new(file, 19)?;
    let mut archive = tar::Builder::new(encoder);
    archive.mode(tar::HeaderMode::Deterministic);
    let root = format!("{}-{}", manifest.id, manifest.version);

    let mut paths = WalkDir::new(directory)
        .follow_links(false)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().is_file())
        .filter_map(|entry| {
            let relative = entry.path().strip_prefix(directory).ok()?.to_path_buf();
            let excluded = relative
                .components()
                .any(|part| matches!(part.as_os_str().to_str(), Some(".git" | "target")))
                || relative == output
                || relative == temporary;
            (!excluded).then(|| (relative, entry.into_path()))
        })
        .collect::<Vec<_>>();
    paths.sort_by(|left, right| left.0.cmp(&right.0));

    for (relative, source) in paths {
        let metadata = fs::symlink_metadata(&source)?;
        if metadata.len() > MAX_PACKAGE_BYTES {
            return Err(format!("{} is too large", source.display()).into());
        }
        let mut header = tar::Header::new_gnu();
        header.set_size(metadata.len());
        header.set_mode(0o644);
        header.set_uid(0);
        header.set_gid(0);
        header.set_mtime(0);
        header.set_cksum();
        let mut input = File::open(&source)?;
        archive.append_data(&mut header, Path::new(&root).join(relative), &mut input)?;
    }
    archive.finish()?;
    let encoder = archive.into_inner()?;
    encoder.finish()?;
    fs::rename(&temporary, &output)?;
    let bytes = fs::read(&output)?;
    if bytes.len() as u64 > MAX_PACKAGE_BYTES {
        fs::remove_file(&output)?;
        return Err("package exceeds the maximum compressed size".into());
    }
    let digest = format!("{:x}", Sha256::digest(&bytes));
    let checksum_name = format!(
        "{}.sha256",
        output
            .file_name()
            .ok_or("output must have a file name")?
            .to_string_lossy()
    );
    fs::write(
        output.with_file_name(checksum_name),
        format!(
            "{digest}  {}\n",
            output.file_name().unwrap().to_string_lossy()
        ),
    )?;
    println!("{}\nsha256 {digest}", output.display());
    Ok(())
}

fn keygen(key_id: &str, output: &Path) -> Result<(), Box<dyn std::error::Error>> {
    if key_id.trim().is_empty() || key_id.chars().any(char::is_whitespace) {
        return Err("key ID must be non-empty and contain no whitespace".into());
    }
    fs::create_dir_all(output)?;
    let signing = SigningKey::generate(&mut OsRng);
    let verifying: VerifyingKey = signing.verifying_key();
    write_private(
        &output.join(format!("{key_id}.private")),
        &STANDARD.encode(signing.to_bytes()),
    )?;
    fs::write(
        output.join(format!("{key_id}.public")),
        format!("{key_id} {}\n", STANDARD.encode(verifying.to_bytes())),
    )?;
    println!("generated {key_id}; keep the .private file secret and offline");
    Ok(())
}

fn write_private(path: &Path, contents: &str) -> io::Result<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        let mut file = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o600)
            .open(path)?;
        writeln!(file, "{contents}")
    }
    #[cfg(not(unix))]
    {
        let mut file = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(path)?;
        writeln!(file, "{contents}")
    }
}
