extern crate structopt;
#[macro_use]
extern crate structopt_derive;

extern crate cargo_metadata;

#[macro_use] extern crate log;
extern crate simple_logger;

use std::process::{exit, Command, Stdio};
use std::ffi::OsString;
use std::path::Path;

use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "cargo remote")]
struct Opts {
    // workaround for "remote" argument when calling "cargo remote"
    _unused: String,
    #[structopt(subcommand)]
    command: Cmd,
    #[structopt(short = "r", long = "remote", help = "remote ssh build server")]
    remote: String
}

#[derive(StructOpt, Debug)]
enum Cmd {
    #[structopt(name="build", help = "Build cargo project remotely and copy back target folder")]
    Build,
}

fn main() {
    simple_logger::init().unwrap();

    let options = Opts::from_args();
    // TODO: add manifest_path option
    let project_metadata = cargo_metadata::metadata(None).unwrap_or_else(|e| {
        error!("Could not read cargo metadata: {}", e);
        exit(-1);
    });

    // for now, assume that there is only one project and find it's root directory
    let (project_dir, project_name) = project_metadata.packages.first().map_or_else(|| {
        error!("No project found.");
        exit(-2);
    }, |project| {
        (
            Path::new(&project.manifest_path).parent().expect("Cargo.toml seems to have no parent directory?"),
            &project.name
        )
    });

    let build_server = options.remote;

    match options.command {
        Build => {
            info!("Transferring sources to build server.");
            // transfer project to build server
            Command::new("rsync")
                .arg("-a")
                .arg("--delete")
                .arg("--info=progress2")
                .arg("--exclude")
                .arg("target")
                .arg(format!("{}/", project_dir.to_string_lossy()))
                .arg(format!("{}:/tmp/remote-build-{}/", build_server, project_name))
                .stdout(Stdio::inherit())
                .stderr(Stdio::inherit())
                .stdin(Stdio::inherit())
                .output()
                .expect("failed to transfer project to build server");

            let build_command = format!("cd /tmp/remote-build-{}/; $HOME/.cargo/bin/cargo build", project_name);

            info!("Starting build process.");
            Command::new("ssh")
                .arg(&build_server)
                .arg(build_command)
                .stdout(Stdio::inherit())
                .stderr(Stdio::inherit())
                .stdin(Stdio::inherit())
                .output()
                .expect("failed to build project");

            info!("Transferring artifacts back to client.");
            Command::new("rsync")
                .arg("-a")
                .arg("--delete")
                .arg("--info=progress2")
                .arg(format!("{}:/tmp/remote-build-{}/target/", build_server, project_name))
                .arg(format!("{}/target/", project_dir.to_string_lossy()))
                .stdout(Stdio::inherit())
                .stderr(Stdio::inherit())
                .stdin(Stdio::inherit())
                .output()
                .expect("failed to transfer built project to client");

        }
    }
}
