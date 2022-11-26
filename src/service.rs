use crate::CliArgs;
use clap::ValueEnum;
use std::fmt;
use std::fmt::Display;

#[derive(Clone)]
pub struct Service {
    pub name: String,
    pub description: String,
    pub after: String,

    pub exec_ctx: ExecutionContext,
    pub install_ctx: InstallContext,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum ServiceType {
    Simple,
    Forking,
    Oneshot,
    Idle,
}

impl Display for ServiceType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ServiceType::Simple => write!(f, "simple"),
            ServiceType::Forking => write!(f, "forking"),
            ServiceType::Oneshot => write!(f, "oneshot"),
            ServiceType::Idle => write!(f, "idle"),
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum RestartPolicy {
    Always,
    OnFailure,
    OnSuccess,
    No,
}

impl Display for RestartPolicy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RestartPolicy::Always => write!(f, "always"),
            RestartPolicy::OnFailure => write!(f, "on-failure"),
            RestartPolicy::OnSuccess => write!(f, "on-success"),
            RestartPolicy::No => write!(f, "no"),
        }
    }
}

#[derive(Clone)]
pub struct ExecutionContext {
    pub exec_start: String,
    pub service_type: ServiceType,
    pub restart_policy: RestartPolicy,
    pub user: Option<String>,
    pub group: Option<String>,
}

#[derive(Clone)]
pub struct InstallContext {
    pub wanted_by: String,
}

impl From<CliArgs> for Service {
    fn from(cli: CliArgs) -> Self {
        Service {
            name: cli.name.unwrap_or(Service::default().name),
            exec_ctx: ExecutionContext {
                exec_start: cli.command,
                service_type: cli.service_type.unwrap_or(ServiceType::Simple),
                restart_policy: cli.restart.unwrap_or(RestartPolicy::No),
                user: cli.user,
                group: cli.group,
            },
            ..Service::default()
        }
    }
}

impl Default for Service {
    fn default() -> Self {
        Service {
            name: "metsysd-42".to_string(),
            description: "This service has been generated with metsysd".to_string(),
            after: "network.target".to_string(),
            exec_ctx: ExecutionContext::default(),
            install_ctx: InstallContext::default(),
        }
    }
}

impl Default for InstallContext {
    fn default() -> Self {
        InstallContext {
            wanted_by: "multi-user.target".to_string(),
        }
    }
}

impl Default for ExecutionContext {
    fn default() -> Self {
        ExecutionContext {
            exec_start: "echo 'hello world'".to_string(),
            service_type: ServiceType::Simple,
            restart_policy: RestartPolicy::No,
            user: None,
            group: None,
        }
    }
}

impl Display for Service {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "[Unit]")?;
        writeln!(f, "Description={}", self.description)?;
        writeln!(f, "After={}", self.after)?;
        writeln!(f, "[Service]")?;
        writeln!(f, "ExecStart={}", self.exec_ctx.exec_start)?;
        writeln!(f, "Type={}", self.exec_ctx.service_type)?;
        writeln!(f, "Restart={}", self.exec_ctx.restart_policy)?;

        if self.exec_ctx.user.is_some() {
            writeln!(f, "User={}", self.exec_ctx.user.as_ref().unwrap())?;
        }

        if self.exec_ctx.group.is_some() {
            writeln!(f, "Group={}", self.exec_ctx.group.as_ref().unwrap())?;
        }

        writeln!(f, "[Install]")?;
        writeln!(f, "WantedBy={}", self.install_ctx.wanted_by)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_service_from_cli() {
        let cli = CliArgs {
            name: Some("test".to_string()),
            command: "echo 'Hello World'".to_string(),
            service_type: Some(ServiceType::Simple),
            ..CliArgs::default()
        };

        let service = Service::from(cli);

        assert_eq!(service.name, "test");
        assert_eq!(service.exec_ctx.exec_start, "echo 'Hello World'");
        assert_eq!(service.exec_ctx.service_type, ServiceType::Simple);
    }

    #[test]
    fn test_service_from_cli_default_service_type() {
        let cli = CliArgs {
            name: Some("test".to_string()),
            command: "echo 'Hello World'".to_string(),
            ..CliArgs::default()
        };

        let service = Service::from(cli);

        assert_eq!(service.name, "test");
        assert_eq!(service.exec_ctx.exec_start, "echo 'Hello World'");
        assert_eq!(service.exec_ctx.service_type, ServiceType::Simple);
    }

    #[test]
    fn test_service_string_from_service() {
        let service = Service {
            name: "test".to_string(),
            description: "This service has been generated with metsysd".to_string(),
            after: "network.target".to_string(),
            exec_ctx: ExecutionContext {
                exec_start: "echo 'Hello World'".to_string(),
                service_type: ServiceType::Simple,
                ..ExecutionContext::default()
            },
            install_ctx: InstallContext {
                wanted_by: "multi-user.target".to_string(),
            },
        };

        let service_file = service.to_string();
        let expected_service_file = "[Unit]
Description=This service has been generated with metsysd
After=network.target
[Service]
ExecStart=echo 'Hello World'
Type=simple
Restart=no
[Install]
WantedBy=multi-user.target
";
        assert_eq!(service_file, expected_service_file)
    }
}
