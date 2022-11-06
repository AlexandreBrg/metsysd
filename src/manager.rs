use crate::service::Service;
use home::home_dir;
use log::{debug, error, info};
use std::fs;
use std::fs::File;
use std::io::{Error, Write};
use std::path::PathBuf;
use std::process::Child;

const SYSTEM_SERVICES_DIR: &str = "/etc/systemd/system";

pub struct Manager {
    /// Services created with this manager will be scoped to user
    is_user: bool,
    /// Custom directory to install services to
    install_dir: PathBuf,
    /// Daemon reload systemd services when done
    with_reload: bool,
}

pub struct ManagerBuilder {
    is_user: bool,
    install_dir: Option<String>,
    with_reload: bool,
}

impl ManagerBuilder {
    pub fn new() -> Self {
        ManagerBuilder {
            is_user: false,
            install_dir: None,
            with_reload: true,
        }
    }

    pub fn set_is_user(mut self, is_user: bool) -> Self {
        self.is_user = is_user;
        self
    }

    pub fn set_install_dir(mut self, install_dir: Option<String>) -> Self {
        self.install_dir = install_dir;
        self
    }

    pub fn set_with_reload(mut self, with_reload: bool) -> Self {
        self.with_reload = with_reload;
        self
    }

    pub fn build(self) -> std::io::Result<Manager> {
        Ok(Manager {
            is_user: self.is_user,
            with_reload: self.with_reload,
            install_dir: match self.install_dir {
                Some(dir) => PathBuf::from(dir),
                None => {
                    if self.is_user {
                        get_user_install_dir()?
                    } else {
                        PathBuf::from(SYSTEM_SERVICES_DIR)
                    }
                }
            },
        })
    }
}

impl Manager {
    pub fn create_service(&self, service: Service) -> std::io::Result<()> {
        debug!("Manager::create_service");

        // We only want to create path when there is no custom directory set
        if self.is_user && self.install_dir == get_user_install_dir()? {
            self.create_user_dir()?;
        }

        let path = self.install_dir.join(format!("{}.service", service.name));
        let mut file = File::create(&path).map_err(|e| {
            error!(
                "Couldn't find '{}' path, are you sure it exists ?",
                self.install_dir.display()
            );
            error!("Hint: You can use --install-dir to specify a custom path");
            e
        })?;
        file.write_all(service.to_string().as_bytes())?;
        debug!(
            "Service file created at {}",
            fs::canonicalize(&path)?.display()
        );
        info!("Service has been installed");

        if self.with_reload {
            self.reload_daemon()?;
        }
        Ok(())
    }

    fn create_user_dir(&self) -> std::io::Result<()> {
        debug!("Manager::create_user_dir");
        fs::create_dir_all(&self.install_dir)?;
        Ok(())
    }

    fn reload_daemon(&self) -> Result<Child, Error> {
        debug!("Manager::reload_daemon");
        let args = if self.is_user {
            vec!["--user", "daemon-reload"]
        } else {
            vec!["daemon-reload"]
        };
        std::process::Command::new("systemctl")
            .args(args)
            .spawn()
            .map_err(|e| {
                error!("Couldn't reload systemd daemon");
                e
            })
    }
}

fn get_user_install_dir() -> std::io::Result<PathBuf> {
    let home_dir = home_dir()
        .ok_or_else(|| Error::new(std::io::ErrorKind::NotFound, "Couldn't find home directory"))?;
    Ok(home_dir.join(".config/systemd/user"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;

    #[test]
    fn test_build_manager() {
        let manager = ManagerBuilder::new().build().unwrap();
        assert_eq!(manager.is_user, false);
        assert_eq!(manager.install_dir, PathBuf::from(SYSTEM_SERVICES_DIR));
    }

    #[test]
    fn test_build_manager_with_user() {
        let manager = ManagerBuilder::new().set_is_user(true).build().unwrap();
        assert_eq!(manager.is_user, true);
        assert_eq!(manager.install_dir, get_user_install_dir().unwrap());
    }

    #[test]
    fn test_build_manager_with_install_dir() {
        let manager = ManagerBuilder::new()
            .set_install_dir(Some("/tmp".to_string()))
            .build()
            .unwrap();
        assert_eq!(manager.is_user, false);
        assert_eq!(manager.install_dir, PathBuf::from("/tmp"));
    }

    #[test]
    fn test_build_manager_with_reload() {
        let manager = ManagerBuilder::new()
            .set_with_reload(false)
            .build()
            .unwrap();
        assert_eq!(manager.is_user, false);
        assert_eq!(manager.install_dir, PathBuf::from(SYSTEM_SERVICES_DIR));
        assert_eq!(manager.with_reload, false);
    }

    #[test]
    fn test_build_manager_with_install_dir_and_user() {
        let manager = ManagerBuilder::new()
            .set_is_user(true)
            .set_install_dir(Some("/tmp".to_string()))
            .build()
            .unwrap();
        assert_eq!(manager.is_user, true);
        assert_eq!(manager.install_dir, PathBuf::from("/tmp"));
    }

    #[test]
    fn test_create_service_content() -> std::io::Result<()> {
        let manager = ManagerBuilder::new()
            .set_install_dir(Some("/tmp".to_string()))
            .set_with_reload(false)
            .build()
            .unwrap();
        let service = Service {
            name: "test_create_servic_tmp".to_string(),
            ..Service::default()
        };

        let create_svc = manager.create_service(service.clone());
        assert_eq!(true, create_svc.is_ok());

        let file = File::open(format!("/tmp/{}.service", &service.name))?;
        let file_content = file.bytes().map(|b| b.unwrap() as char).collect::<String>();
        assert_eq!(file_content, service.to_string());
        Ok(())
    }

    #[test]
    fn test_fail_create_service_content() -> std::io::Result<()> {
        let manager = ManagerBuilder::new()
            .set_install_dir(Some("/tmp/this/path/does/not/exist".to_string()))
            .set_with_reload(false)
            .build()
            .unwrap();
        let service = Service {
            name: "test_create_servic_tmp".to_string(),
            ..Service::default()
        };

        let create_svc = manager.create_service(service.clone());
        assert_eq!(true, create_svc.is_err());
        Ok(())
    }
    #[test]
    fn test_create_user_dir() -> std::io::Result<()> {
        let manager = ManagerBuilder::new()
            .set_install_dir(Some("/tmp/to/create".to_string()))
            .set_with_reload(false)
            .build()
            .unwrap();
        let create_dir = manager.create_user_dir();
        assert_eq!(true, create_dir.is_ok());
        Ok(())
    }
}
