use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct Remote {
    pub host: String,
    pub user: String,
    pub ssh_port: u16,
    pub temp_dir: String,
}

impl Default for Remote {
    fn default() -> Self {
        Self {
            host: String::new(),
            user: String::new(),
            ssh_port: 22,
            temp_dir: "~/remote-builds".to_string(),
        }
    }
}

impl Remote {
    pub fn user_host(&self) -> String {
        format!("{}@{}", self.user, self.host)
    }
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct Config {
    pub remote: Remote,
}

impl Config {
    pub fn new(project_dir: &std::path::Path) -> Result<Self, config::ConfigError> {
        let mut conf = config::Config::default();

        conf.merge(config::File::from_str(
            toml::to_string(&Config::default()).unwrap().as_str(),
            config::FileFormat::Toml,
        ))?;

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
}
