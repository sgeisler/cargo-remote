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
remote = "builds@myserver"
```

### Flags and options
```
cargo-remote 

USAGE:
    cargo remote [FLAGS] [OPTIONS] <command>

FLAGS:
    -c, --copy-back          transfer the target folder back to the local machine
        --help               Prints help information
    -h, --transfer-hidden    transfer hidden files and directories to the build server
    -V, --version            Prints version information

OPTIONS:
        --manifest-path <manifest_path>    Path to the manifest to execute
    -r, --remote <remote>                  remote ssh build server

ARGS:
    <command>    cargo command that will be executed remotely
```


## How to install
```bash
git clone https://github.com/sgeisler/cargo-remote.git
cd cargo-remote
cargo install
```
