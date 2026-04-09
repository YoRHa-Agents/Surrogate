#[cfg(target_os = "macos")]
mod platform {
    use anyhow::{Context, Result, bail};
    use std::process::Command;

    pub fn enable_http_proxy(host: &str, port: u16) -> Result<()> {
        let port_str = port.to_string();
        for service in active_network_services()? {
            networksetup(&["-setwebproxy", &service, host, &port_str])?;
            networksetup(&["-setsecurewebproxy", &service, host, &port_str])?;
            networksetup(&["-setwebproxystate", &service, "on"])?;
            networksetup(&["-setsecurewebproxystate", &service, "on"])?;
        }
        Ok(())
    }

    pub fn enable_socks_proxy(host: &str, port: u16) -> Result<()> {
        let port_str = port.to_string();
        for service in active_network_services()? {
            networksetup(&["-setsocksfirewallproxy", &service, host, &port_str])?;
            networksetup(&["-setsocksfirewallproxystate", &service, "on"])?;
        }
        Ok(())
    }

    pub fn enable_all_proxies(host: &str, port: u16) -> Result<()> {
        enable_http_proxy(host, port)?;
        enable_socks_proxy(host, port)?;
        Ok(())
    }

    pub fn disable_all_proxies() -> Result<()> {
        for service in active_network_services()? {
            networksetup(&["-setwebproxystate", &service, "off"])?;
            networksetup(&["-setsecurewebproxystate", &service, "off"])?;
            networksetup(&["-setsocksfirewallproxystate", &service, "off"])?;
        }
        Ok(())
    }

    pub fn is_proxy_enabled() -> Result<bool> {
        let services = active_network_services()?;
        let Some(service) = services.first() else {
            return Ok(false);
        };
        let output = Command::new("networksetup")
            .args(["-getwebproxy", service])
            .output()
            .context("failed to query web proxy state")?;
        let stdout = String::from_utf8_lossy(&output.stdout);
        Ok(stdout.contains("Enabled: Yes"))
    }

    fn active_network_services() -> Result<Vec<String>> {
        let output = Command::new("networksetup")
            .arg("-listallnetworkservices")
            .output()
            .context("failed to list network services")?;
        if !output.status.success() {
            bail!(
                "networksetup -listallnetworkservices failed: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        }
        let stdout = String::from_utf8_lossy(&output.stdout);
        let services: Vec<String> = stdout
            .lines()
            .skip(1)
            .filter(|line| !line.starts_with('*'))
            .map(|line| line.trim().to_string())
            .filter(|line| !line.is_empty())
            .collect();
        Ok(services)
    }

    fn networksetup(args: &[&str]) -> Result<()> {
        let output = Command::new("networksetup")
            .args(args)
            .output()
            .with_context(|| format!("networksetup {} failed", args.join(" ")))?;
        if !output.status.success() {
            bail!(
                "networksetup {} failed: {}",
                args.join(" "),
                String::from_utf8_lossy(&output.stderr)
            );
        }
        Ok(())
    }
}

#[cfg(not(target_os = "macos"))]
mod platform {
    use anyhow::{Result, bail};

    pub fn enable_http_proxy(_host: &str, _port: u16) -> Result<()> {
        bail!("system proxy management is only supported on macOS")
    }

    pub fn enable_socks_proxy(_host: &str, _port: u16) -> Result<()> {
        bail!("system proxy management is only supported on macOS")
    }

    pub fn enable_all_proxies(_host: &str, _port: u16) -> Result<()> {
        bail!("system proxy management is only supported on macOS")
    }

    pub fn disable_all_proxies() -> Result<()> {
        bail!("system proxy management is only supported on macOS")
    }

    pub fn is_proxy_enabled() -> Result<bool> {
        bail!("system proxy management is only supported on macOS")
    }
}

pub use platform::*;

#[cfg(test)]
mod tests {
    #[test]
    fn platform_functions_are_accessible() {
        let _ = super::is_proxy_enabled();
    }

    #[cfg(not(target_os = "macos"))]
    #[test]
    fn non_macos_returns_error() {
        let err = super::enable_http_proxy("127.0.0.1", 8080)
            .expect_err("should fail on non-macOS");
        assert!(err.to_string().contains("only supported on macOS"));

        let err = super::disable_all_proxies()
            .expect_err("should fail on non-macOS");
        assert!(err.to_string().contains("only supported on macOS"));
    }
}
