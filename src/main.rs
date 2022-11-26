use clap::Parser;
use log::{error, info};

use crate::manager::{Manager, ManagerBuilder};
use crate::service::{RestartPolicy, Service, ServiceType};

mod manager;
mod service;

#[derive(Parser, Debug, Clone)]
#[command(author, version, about)]
struct CliArgs {
    /// Name of the service you want to create
    #[arg(short, long)]
    name: Option<String>,

    /// The command which will be executed when running the service
    command: String,

    /// Defines the kind of service you want to run
    ///
    /// See https://www.freedesktop.org/software/systemd/man/systemd.service.html#Type= for more information
    #[arg(long, value_enum)]
    service_type: Option<ServiceType>,

    /// Defines the restart policy of the service
    #[arg(long, value_enum)]
    restart: Option<RestartPolicy>,

    /// Specify the user running the service (it must exist)
    #[arg(long)]
    user: Option<String>,

    /// Specify the group running the service (it must exist)
    #[arg(long)]
    group: Option<String>,

    /// Defines your service as a service from your user (rootless). The service will start when
    /// the user runs a session on the host.
    #[arg(long)]
    is_user: bool,

    /// Define the directory to which the service should be installed. Only use when you know what you're doing.
    #[arg(long)]
    install_dir: Option<String>,

    /// Run daemon-reload when service has been created
    #[arg(long, default_value = "true")]
    daemon_reload: bool,

    /// Output the generated service in the console, instead of creating it
    #[arg(short, long)]
    dry_run: bool,
}

impl Default for CliArgs {
    fn default() -> Self {
        CliArgs {
            name: Some("test".to_string()),
            command: "echo 'hello world'".to_string(),
            service_type: None,
            is_user: false,
            install_dir: None,
            daemon_reload: true,
            dry_run: false,
            restart: None,
            user: None,
            group: None,
        }
    }
}

fn main() -> std::io::Result<()> {
    let cli: CliArgs = CliArgs::parse();
    env_logger::init_from_env(
        env_logger::Env::default().filter_or(env_logger::DEFAULT_FILTER_ENV, "info"),
    );
    let service: Service = cli.clone().into();
    let manager: Manager = ManagerBuilder::new()
        .set_is_user(cli.is_user)
        .set_install_dir(cli.install_dir)
        .set_with_reload(cli.daemon_reload)
        .build()?;

    if cli.dry_run {
        info!("Dry run enabled, not creating service");
        info!("Service file contents:\n{}", service);
        return Ok(());
    }

    match manager.create_service(service) {
        Ok(_) => info!("Service created successfully"),
        Err(e) => error!("Error creating service: {}", e),
    }
    Ok(())
}
