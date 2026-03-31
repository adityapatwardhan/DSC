use clap::{Parser, Subcommand};
use oci::client::Client;
use oci::secrets::RegistryAuth;
use oci::Reference;
use std::path::{Path, PathBuf};

#[derive(Debug, Parser)]
#[command(
    name = "oci-discover",
    version,
    about = "Discover, find, and download DSC OCI artifacts"
)]
struct Cli {
    /// OCI repository URL (for example: ghcr.io/owner/repository)
    #[arg(long = "repository-url")]
    repository_url: String,

    /// Optional username for private repository authentication
    #[arg(long)]
    username: Option<String>,

    /// Optional token/password for private repository authentication
    #[arg(long)]
    token: Option<String>,

    #[command(subcommand)]
    operation: Operation,
}

#[derive(Debug, Subcommand)]
enum Operation {
    /// Discover DSC manifests from an OCI repository and print available artifact names
    Discover,

    /// Find a specific artifact in the repository
    Find {
        /// Artifact name/tag to search for
        #[arg(long)]
        artifact: String,
    },

    /// Download a specific artifact from the repository
    Download {
        /// Artifact name/tag to download
        #[arg(long)]
        artifact: String,

        /// Output directory for the downloaded artifact payload
        #[arg(long, default_value = ".")]
        output: PathBuf,
    },
}

#[tokio::main]
async fn main() -> Result<(), String> {
    let cli = Cli::parse();

    validate_auth(cli.username.as_deref(), cli.token.as_deref())?;

    match cli.operation {
        Operation::Discover => discover(&cli).await,
        Operation::Find { artifact } => find(&cli, &artifact).await,
        Operation::Download { artifact, output } => download(&cli, &artifact, &output).await,
    }
}

fn validate_auth(username: Option<&str>, token: Option<&str>) -> Result<(), String> {
    match (username, token) {
        (None, None) => Ok(()),
        (Some(_), Some(_)) => Ok(()),
        _ => Err("when using authentication, provide both --username and --token".to_string()),
    }
}

fn registry_auth(cli: &Cli) -> RegistryAuth {
    match (cli.username.as_ref(), cli.token.as_ref()) {
        (Some(username), Some(token)) => RegistryAuth::Basic(username.clone(), token.clone()),
        _ => RegistryAuth::Anonymous,
    }
}

fn reference_for_artifact(repository_url: &str, artifact: &str) -> Result<Reference, String> {
    let reference = if artifact.starts_with("sha256:") {
        format!("{repository_url}@{artifact}")
    } else {
        format!("{repository_url}:{artifact}")
    };

    reference
        .parse::<Reference>()
        .map_err(|err| format!("invalid reference '{reference}': {err}"))
}

async fn discover(cli: &Cli) -> Result<(), String> {
    let client = Client::default();
    let auth = registry_auth(cli);

    let repository_ref = cli
        .repository_url
        .parse::<Reference>()
        .map_err(|err| format!("invalid repository URL '{}': {err}", cli.repository_url))?;

    let tags = client
        .list_tags(&repository_ref, &auth, None, None)
        .await
        .map_err(|err| format!("failed to list tags: {err}"))?;

    for tag in tags.tags {
        println!("{tag}");
    }

    Ok(())
}

async fn find(cli: &Cli, artifact: &str) -> Result<(), String> {
    let client = Client::default();
    let auth = registry_auth(cli);
    let reference = reference_for_artifact(&cli.repository_url, artifact)?;

    let digest = client
        .fetch_manifest_digest(&reference, &auth)
        .await
        .map_err(|err| {
            format!(
                "failed to find artifact '{artifact}' in '{}': {err}",
                cli.repository_url
            )
        })?;

    println!("artifact={artifact}");
    println!("repository={}", cli.repository_url);
    println!("digest={digest}");

    Ok(())
}

async fn download(cli: &Cli, artifact: &str, output: &Path) -> Result<(), String> {
    let client = Client::default();
    let auth = registry_auth(cli);
    let reference = reference_for_artifact(&cli.repository_url, artifact)?;

    tokio::fs::create_dir_all(output)
        .await
        .map_err(|err| format!("failed to create output directory '{}': {err}", output.display()))?;

    let (manifest, manifest_digest) = client
        .pull_image_manifest(&reference, &auth)
        .await
        .map_err(|err| {
            format!(
                "failed to pull manifest for artifact '{artifact}' in '{}': {err}",
                cli.repository_url
            )
        })?;

    println!("manifestDigest={manifest_digest}");

    for (index, layer) in manifest.layers.iter().enumerate() {
        let digest_safe = layer.digest.replace(':', "_");
        let file_name = format!("layer-{index:02}-{digest_safe}.blob");
        let file_path = output.join(file_name);

        let file = tokio::fs::File::create(&file_path)
            .await
            .map_err(|err| format!("failed to create '{}': {err}", file_path.display()))?;

        client
            .pull_blob(&reference, layer, file)
            .await
            .map_err(|err| format!("failed to download layer '{}': {err}", layer.digest))?;

        println!(
            "downloaded layer digest={} mediaType={} size={} path={}",
            layer.digest,
            layer.media_type,
            layer.size,
            file_path.display()
        );
    }

    Ok(())
}
