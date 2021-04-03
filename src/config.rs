use serde::Deserialize;

#[derive(Debug, Clone)]
pub struct Remote {
    pub name: String,
    pub host: String,
    pub ssh_port: u16,
    pub temp_dir: String,
}

#[derive(Debug, Deserialize)]
struct PartialRemote {
    pub name: Option<String>,
    pub host: String,
    pub ssh_port: Option<u16>,
    pub temp_dir: Option<String>,
}

impl Default for Remote {
    fn default() -> Self {
        Self {
            name: String::new(),
            host: String::new(),
            ssh_port: 22,
            temp_dir: "~/remote-builds".to_string(),
        }
    }
}

impl From<PartialRemote> for Remote {
    fn from(minimal_remote: PartialRemote) -> Self {
        let default = Remote::default();
        let name = minimal_remote.name.unwrap_or(default.name);
        let ssh_port = minimal_remote.ssh_port.unwrap_or(default.ssh_port);
        let temp_dir = minimal_remote.temp_dir.unwrap_or(default.temp_dir);
        Remote {
            name,
            host: minimal_remote.host,
            ssh_port,
            temp_dir,
        }
    }
}

impl<'de> Deserialize<'de> for Remote {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        PartialRemote::deserialize(deserializer).map(Self::from)
    }
}

#[derive(Debug, Default, Deserialize)]
pub struct Config {
    #[serde(rename = "remote")]
    remotes: Option<Vec<Remote>>,
}

impl Config {
    pub fn new(project_dir: &std::path::Path) -> Result<Self, config::ConfigError> {
        let mut conf = config::Config::new();

        if let Some(config_file) = xdg::BaseDirectories::with_prefix("cargo-remote")
            .ok()
            .and_then(|base| base.find_config_file("cargo-remote.toml"))
        {
            conf.merge(config::File::from(config_file))?;
        }

        let project_config = project_dir.join(".cargo-remote.toml");
        if project_config.is_file() {
            conf.merge(config::File::from(project_config))?;
        }

        conf.try_into()
    }

    pub fn get_remote(&self, opts: &crate::RemoteOpts) -> Option<Remote> {
        let remotes: Vec<_> = self.remotes.clone().unwrap_or_default();
        let config_remote = match &opts.name {
            Some(remote_name) => remotes
                .into_iter()
                .find(|remote| remote.name == *remote_name),
            None => remotes.into_iter().next(),
        };

        let blueprint_remote = match (config_remote, opts.host.is_some()) {
            (Some(config_remote), _) => config_remote,
            (None, true) => Remote::default(),
            (None, false) => return None,
        };

        Some(Remote {
            name: opts.name.clone().unwrap_or(blueprint_remote.name),
            host: opts.host.clone().unwrap_or(blueprint_remote.host),
            ssh_port: opts.ssh_port.clone().unwrap_or(blueprint_remote.ssh_port),
            temp_dir: opts.temp_dir.clone().unwrap_or(blueprint_remote.temp_dir),
        })
    }
}
