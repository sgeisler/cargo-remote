# Cargo Remote

***Use with caution, I didn't test this software well and it is a really ugly 
hack (at least for now).***

## Why is it useful
One big annoyance when working on rust projects on my notebook are the compile
times. Since I'm using rust nightly for some of my projects I have to recompile
rather often. Currently there seem to be no good remote-build integrations for
rust, so I decided to build one my own.

## Planned capabilities
This first version is very dumb (could have been a bash script), but I intend to
enhance it to a point where it detects compatibility between local and remote
versions, allows (nearly) all cargo commands and maybe even load distribution
over multiple machines.

## Current capabilities
For now only `cargo remote --remote=user@server build` works: it copies the
current project to a temporary directory on the remote server, calls
`cargo build` remotely and copies back the resulting target folder. This assumes
that server and client are running the same rust version and have the same
processor architecture. On the client `ssh` and `rsync` need to be installed.

## How to install
```bash
git clone https://github.com/sgeisler/cargo-remote.git
cd cargo-remote
cargo install
```