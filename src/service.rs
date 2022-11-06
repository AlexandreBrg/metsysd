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

#[derive(Clone)]
pub struct ExecutionContext {
    pub exec_start: String,
    pub service_type: ServiceType,
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
                service_type: cli
                    .service_type
                    .unwrap_or(ExecutionContext::default().service_type),
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

impl Default for ExecutionContext {
    fn default() -> Self {
        ExecutionContext {
            service_type: ServiceType::Simple,
            exec_start: "echo 'Hello World'".to_string(),
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

impl Display for Service {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "[Unit]")?;
        writeln!(f, "Description={}", self.description)?;
        writeln!(f, "After={}", self.after)?;
        writeln!(f, "[Service]")?;
        writeln!(f, "ExecStart={}", self.exec_ctx.exec_start)?;
        writeln!(f, "Type={}", self.exec_ctx.service_type)?;
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
[Install]
WantedBy=multi-user.target
";
        assert_eq!(service_file, expected_service_file)
    }
}
