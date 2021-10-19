use std::collections::hash_map::DefaultHasher;
use std::fs::canonicalize;
use std::hash::{Hash, Hasher};
use std::path::PathBuf;
use std::process::{exit, Command, Stdio};
use structopt::StructOpt;

use log::{error, info};

mod config;

const PROGRESS_FLAG: &str = "--info=progress2";

#[derive(StructOpt, Debug)]
pub struct RemoteOpts {
    /// The name of the remote specified in the config
    #[structopt(short = "r", long = "remote")]
    name: Option<String>,

    /// Remote ssh build server with user or the name of the ssh entry
    #[structopt(short = "H", long = "remote-host")]
    host: Option<String>,

    /// The ssh port to communicate with the build server
    #[structopt(short = "p", long = "remote-ssh-port")]
    ssh_port: Option<u16>,

    /// The directory where cargo builds the project
    #[structopt(short, long = "remote-temp-dir")]
    temp_dir: Option<String>,

    #[structopt(
        short = "e",
        long = "env",
        help = "Environment profile. default_value = /etc/profile"
    )]
    env: Option<String>,
}

#[derive(StructOpt, Debug)]
#[structopt(name = "cargo-remote", bin_name = "cargo")]
enum Opts {
    #[structopt(name = "remote")]
    Remote {
        #[structopt(flatten)]
        remote_opts: RemoteOpts,

        #[structopt(
            short = "b",
            long = "build-env",
            help = "Set remote environment variables. RUST_BACKTRACE, CC, LIB, etc. ",
            default_value = "RUST_BACKTRACE=1"
        )]
        build_env: String,

        #[structopt(
            short = "d",
            long = "rustup-default",
            help = "Rustup default (stable|beta|nightly)",
            default_value = "stable"
        )]
        rustup_default: String,

        #[structopt(
            short = "c",
            long = "copy-back",
            help = "Transfer the target folder or specific file from that folder back to the local machine"
        )]
        copy_back: Option<Option<String>>,

        #[structopt(
            long = "no-copy-lock",
            help = "don't transfer the Cargo.lock file back to the local machine"
        )]
        no_copy_lock: bool,

        #[structopt(
            long = "manifest-path",
            help = "Path to the manifest to execute",
            default_value = "Cargo.toml",
            parse(from_os_str)
        )]
        manifest_path: PathBuf,

        #[structopt(
            short = "h",
            long = "transfer-hidden",
            help = "Transfer hidden files and directories to the build server"
        )]
        hidden: bool,

        #[structopt(help = "cargo command that will be executed remotely")]
        command: String,

        #[structopt(
            short = "w",
            long = "working-directory",
            help = "The working directory to copy files from. Default is your workspace root."
        )]
        working_directory: Option<String>,

        #[structopt(
            help = "cargo options and flags that will be applied remotely",
            name = "remote options"
        )]
        options: Vec<String>,
    },
}

fn main() {
    simple_logger::init().unwrap();

    let Opts::Remote {
        remote_opts,
        build_env,
        rustup_default,
        copy_back,
        no_copy_lock,
        manifest_path,
        hidden,
        command,
        working_directory,
        options,
    } = Opts::from_args();

    let mut metadata_cmd = cargo_metadata::MetadataCommand::new();
    metadata_cmd.manifest_path(manifest_path).no_deps();

    let project_metadata = metadata_cmd.exec().unwrap();

    let project_root = match working_directory {
        Some(path) => canonicalize(PathBuf::from(path))
            .expect("The provided working directory does not exist or has an error."),
        None => project_metadata.workspace_root.clone(),
    };
    info!(
        "Workspace root: {:?}",
        project_metadata.workspace_root.clone()
    );
    info!("Project root: {:?}", project_root);

    let diff_from_project_root = project_metadata
        .workspace_root
        .strip_prefix(project_root.clone())
        .expect("Working directory should be an ancestor of the workspace root")
        .to_owned();

    let conf = match config::Config::new(&project_root) {
        Ok(conf) => conf,
        Err(error) => {
            error!("{}", error);
            exit(-3);
        }
    };

    let remote = match conf.get_remote(&remote_opts) {
        Some(remote) => remote,
        None => {
            error!("No remote build server was defined (use config file or the --remote flags)");
            exit(4);
        }
    };

    let build_server = remote.host;

    // generate a unique build path by using the hashed project dir as folder on the remote machine
    let mut hasher = DefaultHasher::new();
    project_root.hash(&mut hasher);
    let build_path = format!("{}/{}", remote.temp_dir, hasher.finish());

    let mut build_workspace = PathBuf::from(build_path.clone());
    build_workspace.push(diff_from_project_root.clone());

    let mut project_workspace = project_root.clone();
    project_workspace.push(diff_from_project_root);

    info!("Transferring sources to build server.");
    // transfer project to build server
    let mut rsync_to = Command::new("rsync");
    rsync_to
        .arg("-a".to_owned())
        .arg("--delete")
        .arg("--compress")
        .arg("-e")
        .arg(format!("ssh -p {}", remote.ssh_port))
        .arg(PROGRESS_FLAG)
        .arg("--exclude")
        .arg("target");

    if !hidden {
        rsync_to.arg("--exclude").arg(".*");
    }

    rsync_to
        .arg("--rsync-path")
        .arg("mkdir -p remote-builds && rsync")
        .arg(format!("{}/", project_root.to_string_lossy()))
        .arg(format!("{}:{}", build_server, build_path))
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .stdin(Stdio::inherit())
        .output()
        .unwrap_or_else(|e| {
            error!("Failed to transfer project to build server (error: {})", e);
            exit(-4);
        });
    info!("Build ENV: {:?}", build_env);
    info!("Environment profile: {:?}", remote.env);
    info!("Build path: {:?}", build_path);
    let build_command = format!(
        "source {}; rustup default {}; cd {}; {} cargo {} {}",
        remote.env,
        rustup_default,
        build_workspace.to_string_lossy(),
        build_env,
        command,
        options.join(" ")
    );

    info!("Starting build process.");
    let output = Command::new("ssh")
        .args(&["-p", &remote.ssh_port.to_string()])
        .arg("-t")
        .arg(&build_server)
        .arg(build_command)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .stdin(Stdio::inherit())
        .output()
        .unwrap_or_else(|e| {
            error!("Failed to run cargo command remotely (error: {})", e);
            exit(-5);
        });

    if let Some(file_name) = copy_back {
        info!("Transferring artifacts back to client.");
        let file_name = file_name.unwrap_or_else(String::new);
        Command::new("rsync")
            .arg("-a")
            .arg("--delete")
            .arg("--compress")
            .arg("-e")
            .arg(format!("ssh -p {}", remote.ssh_port))
            .arg(PROGRESS_FLAG)
            .arg(format!(
                "{}:{}/target/{}",
                build_server,
                build_workspace.to_string_lossy(),
                file_name
            ))
            .arg(format!(
                "{}/target/{}",
                project_workspace.to_string_lossy(),
                file_name
            ))
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .stdin(Stdio::inherit())
            .output()
            .unwrap_or_else(|e| {
                error!(
                    "Failed to transfer target back to local machine (error: {})",
                    e
                );
                exit(-6);
            });
    }

    if !no_copy_lock {
        info!("Transferring Cargo.lock file back to client.");
        Command::new("rsync")
            .arg("-a")
            .arg("--delete")
            .arg("--compress")
            .arg("-e")
            .arg(format!("ssh -p {}", remote.ssh_port))
            .arg(PROGRESS_FLAG)
            .arg(format!(
                "{}:{}/Cargo.lock",
                build_server,
                build_workspace.to_string_lossy()
            ))
            .arg(format!(
                "{}/Cargo.lock",
                project_workspace.to_string_lossy()
            ))
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .stdin(Stdio::inherit())
            .output()
            .unwrap_or_else(|e| {
                error!(
                    "Failed to transfer Cargo.lock back to local machine (error: {})",
                    e
                );
                exit(-7);
            });
    }

    if !output.status.success() {
        exit(output.status.code().unwrap_or(1))
    }
}
