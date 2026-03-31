# OCI Discover Extension (Rust)

This folder contains a DSC extension manifest and a Rust CLI scaffold for
`Microsoft.DSC/Discover`.

## Extension Manifest

- File: `oci-discover.dsc.extension.json`
- Type: `Microsoft.DSC/Discover`
- Executable: `oci-discover`

## CLI Operations

The CLI supports one operation at a time using clap subcommands:

- `discover`
- `find --artifact <name>`
- `download --artifact <name> [--output <dir>]`

Shared parameters:

- `--repository-url <url>` (required)
- `--username <name>` (optional)
- `--token <token>` (optional)

Authentication is optional. If either `--username` or `--token` is provided,
both must be provided.

## Examples

```powershell
cargo run -- --repository-url ghcr.io/example/dsc discover
cargo run -- --repository-url ghcr.io/example/dsc find --artifact demo-artifact:latest
cargo run -- --repository-url ghcr.io/example/dsc --username user --token token123 download --artifact demo-artifact:latest --output ./out
```
