use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

const LABEL: &str = "com.knothhe.scrollsplit";

pub fn install() -> Result<(), String> {
    let path = plist_path()?;
    let executable = env::current_exe()
        .map_err(|error| format!("cannot locate current executable: {error}"))?
        .canonicalize()
        .map_err(|error| format!("cannot resolve executable path: {error}"))?;

    fs::create_dir_all(
        path.parent()
            .ok_or_else(|| format!("invalid LaunchAgent path: {}", path.display()))?,
    )
    .map_err(|error| format!("cannot create LaunchAgents directory: {error}"))?;
    fs::write(&path, render_plist(&executable))
        .map_err(|error| format!("cannot write {}: {error}", path.display()))?;

    let _ = bootout();
    run_launchctl(&["bootstrap", &domain(), path_string(&path)?.as_str()])?;
    println!("Installed and started {}", path.display());
    Ok(())
}

pub fn uninstall() -> Result<(), String> {
    let path = plist_path()?;
    let _ = bootout();
    match fs::remove_file(&path) {
        Ok(()) => println!("Removed {}", path.display()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            println!("Service is not installed")
        }
        Err(error) => return Err(format!("cannot remove {}: {error}", path.display())),
    }
    Ok(())
}

pub fn start() -> Result<(), String> {
    let path = plist_path()?;
    if !path.exists() {
        return Err("service is not installed; run `scrollsplit install-service`".to_owned());
    }

    let domain = domain();
    let target = target();
    let result = run_launchctl(&["bootstrap", &domain, path_string(&path)?.as_str()]);
    if result.is_err() {
        run_launchctl(&["kickstart", "-k", &target])?;
    }
    println!("Started {LABEL}");
    Ok(())
}

pub fn stop() -> Result<(), String> {
    bootout()?;
    println!("Stopped {LABEL}");
    Ok(())
}

pub fn restart() -> Result<(), String> {
    let _ = bootout();
    start()
}

pub fn status() -> Result<(), String> {
    let output = launchctl_output(&["print", &target()])?;
    if output.status.success() {
        print!("{}", String::from_utf8_lossy(&output.stdout));
        Ok(())
    } else {
        Err(format!("{LABEL} is not running"))
    }
}

fn bootout() -> Result<(), String> {
    run_launchctl(&["bootout", &target()])
}

fn run_launchctl(args: &[&str]) -> Result<(), String> {
    let output = launchctl_output(args)?;
    if output.status.success() {
        Ok(())
    } else {
        let message = String::from_utf8_lossy(&output.stderr).trim().to_owned();
        Err(if message.is_empty() {
            format!("launchctl {} failed", args.join(" "))
        } else {
            message
        })
    }
}

fn launchctl_output(args: &[&str]) -> Result<Output, String> {
    Command::new("/bin/launchctl")
        .args(args)
        .output()
        .map_err(|error| format!("cannot run launchctl: {error}"))
}

fn domain() -> String {
    format!("gui/{}", unsafe { libc::geteuid() })
}

fn target() -> String {
    format!("{}/{LABEL}", domain())
}

fn plist_path() -> Result<PathBuf, String> {
    let home = env::var_os("HOME").ok_or_else(|| "HOME is not set".to_owned())?;
    Ok(PathBuf::from(home)
        .join("Library")
        .join("LaunchAgents")
        .join(format!("{LABEL}.plist")))
}

fn path_string(path: &Path) -> Result<String, String> {
    path.to_str()
        .map(ToOwned::to_owned)
        .ok_or_else(|| format!("path is not valid UTF-8: {}", path.display()))
}

fn render_plist(executable: &Path) -> String {
    let executable = xml_escape(&executable.to_string_lossy());
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>Label</key>
  <string>{LABEL}</string>
  <key>ProgramArguments</key>
  <array>
    <string>{executable}</string>
    <string>run</string>
  </array>
  <key>RunAtLoad</key>
  <true/>
  <key>KeepAlive</key>
  <true/>
  <key>ProcessType</key>
  <string>Interactive</string>
</dict>
</plist>
"#
    )
}

fn xml_escape(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::render_plist;

    #[test]
    fn plist_contains_label_command_and_escaped_path() {
        let plist = render_plist(Path::new("/tmp/a&b/scrollsplit"));
        assert!(plist.contains("<string>com.knothhe.scrollsplit</string>"));
        assert!(plist.contains("<string>/tmp/a&amp;b/scrollsplit</string>"));
        assert!(plist.contains("<string>run</string>"));
        assert!(plist.contains("<key>RunAtLoad</key>"));
    }
}
