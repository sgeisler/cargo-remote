# Cargo Remote

***Use with caution, I didn't test this software well and it is a really hacky
(at least for now). If you want to test it please create a VM or at least a separate
user on your build host***

## Why I built it
One big annoyance when working on rust projects on my notebook are the compile
times. Since I'm using rust nightly for some of my projects I have to recompile
rather often. Currently there seem to be no good remote-build integrations for
rust, so I decided to build one my own.

## Planned capabilities
This first version is very simple (could have been a bash script), but I intend to
enhance it to a point where it detects compatibility between local and remote
versions, allows (nearly) all cargo commands and maybe even load distribution
over multiple machines.

## Usage
For now only `cargo remote [FLAGS] [OPTIONS] <command>` works: it copies the
current project to a temporary directory (`~/remote-builds/<project_name>`) on
the remote server, calls `cargo <command>` remotely and optionally (`-c`) copies
back the resulting target folder. This assumes that server and client are running
the same rust version and have the same processor architecture. On the client `ssh`
and `rsync` need to be installed.

If you want to pass remote flags you have to end the options/flags section using
`--`. E.g. to build in release mode and copy back the result use:
```bash
cargo remote -c -- build --release
```

### Configuration
You can place a config file called `.cargo-remote.toml` in the same directory as your
`Cargo.toml` or at `~/.config/cargo-remote/cargo-remote.toml`. There you can define a
default remote build host and user. It can be overridden by the `-r` flag.

Example config file:
```toml
[[remote]]
name = "myRemote" # Not needed for a single remote
host = "myUser@myServer" # Could also be a ssh config entry
ssh_port = 42 # defaults to 22
temp_dir = "~/rust" # Default is "~/remote-builds"
env = "~/.profile" # Default is "/etc/profile"
```

### Flags and options

```
USAGE:
    cargo remote [FLAGS] [OPTIONS] <command> [remote options]...

FLAGS:
        --help               Prints help information
    -h, --transfer-hidden    Transfer hidden files and directories to the build server
        --no-copy-lock       don't transfer the Cargo.lock file back to the local machine
    -V, --version            Prints version information

OPTIONS:
    -b, --build-env <build_env>              Set remote environment variables. RUST_BACKTRACE, CC, LIB, etc.  [default:
                                             RUST_BACKTRACE=1]
    -c, --copy-back <copy_back>              Transfer the target folder or specific file from that folder back to the
                                             local machine
    -e, --env <env>                          Environment profile. default_value = /etc/profile
    -H, --remote-host <host>                 Remote ssh build server with user or the name of the ssh entry
        --manifest-path <manifest_path>      Path to the manifest to execute [default: Cargo.toml]
    -r, --remote <name>                      The name of the remote specified in the config
    -d, --rustup-default <rustup_default>    Rustup default (stable|beta|nightly) [default: stable]
    -p, --remote-ssh-port <ssh_port>         The ssh port to communicate with the build server
    -t, --remote-temp-dir <temp_dir>         The directory where cargo builds the project

ARGS:
    <command>              cargo command that will be executed remotely
    <remote options>...    cargo options and flags that will be applied remotely
```


## How to install
```bash
git clone https://github.com/sgeisler/cargo-remote
cargo install --path cargo-remote/
```

### MacOS Problems
It was reported that the `rsync` version shipped with MacOS doesn't support the progress flag and thus fails when
`cargo-remote` tries to use it. You can install a newer version by running
```bash
brew install rsync
```
See also [#10](https://github.com/sgeisler/cargo-remote/issues/10).
