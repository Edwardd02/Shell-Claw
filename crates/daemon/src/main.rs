mod config;
mod diagnostics;
mod ipc;
mod memory;
mod model;
mod scheduler;

use config::DaemonConfig;
use ipc::server::IpcServer;
use std::path::PathBuf;
use std::process::{Child, Command};
use std::time::{Duration, Instant};
use tracing::info;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();

    match args.first().map(String::as_str) {
        Some("daemon") => run_daemon().await,
        Some("start") => cmd_start(&args),
        Some("stop") => cmd_stop(),
        Some("status") => cmd_status(),
        Some("log") => cmd_log(&args),
        Some("setup") => cmd_setup(&args),
        Some("version") | Some("-V") | Some("--version") => {
            println!("shellclaw {}", env!("CARGO_PKG_VERSION"));
            Ok(())
        }
        Some("help") | Some("-h") | Some("--help") => {
            print_usage();
            Ok(())
        }
        _ => {
            print_usage();
            Ok(())
        }
    }
}

fn print_usage() {
    println!("ShellClaw — 本地优先的智能终端补全");
    println!();
    println!("用法:");
    println!("  shellclaw daemon         前台运行 daemon(通常由服务管理器调用)");
    println!("  shellclaw start          后台启动 daemon");
    println!("  shellclaw stop           停止 daemon");
    println!("  shellclaw status         查看 daemon 状态");
    println!("  shellclaw log on|off     开启/关闭文件日志(持久化)");
    println!("  shellclaw setup PATH     幂等安装 Zsh hook 并启动 daemon");
    println!("  shellclaw help           显示帮助");
}

/// 前台运行 daemon(供 launchd/systemd 或 shellclaw start 的后台进程调用)。
async fn run_daemon() -> std::io::Result<()> {
    let config = DaemonConfig::load();
    let _guard = DaemonPidFile::create()?;

    // 仅当 log 开关开启时才初始化为文件日志。
    if config.log_enabled {
        diagnostics::init(&config.log_path);
        info!("daemon logging to {:?}", config.log_path);
    } else {
        tracing_subscriber::fmt()
            .with_max_level(tracing::Level::WARN)
            .with_writer(std::io::stderr)
            .init();
        info!("daemon started (logging disabled; use 'shellclaw log on')");
    }

    info!("ShellClaw daemon starting");
    info!("Socket path: {:?}", config.socket_path);

    let server = IpcServer::new(config.socket_path.clone());

    tokio::spawn(async move {
        scheduler::global().run().await;
    });

    if let Err(e) = server.run().await {
        tracing::error!("IPC server error: {}", e);
        return Err(std::io::Error::other(e.to_string()));
    }

    diagnostics::flush();
    Ok(())
}

/// 后台启动 daemon(当前可执行文件自身,以 daemon 子命令拉起)。
fn cmd_start(_args: &[String]) -> std::io::Result<()> {
    // 若已在运行
    if daemon_running()? {
        println!("shellclaw: already running");
        return Ok(());
    }
    let exe = std::env::current_exe()?;
    let child = Command::new(exe)
        .arg("daemon")
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn();
    match child {
        Ok(mut child) => wait_for_daemon_start(&mut child),
        Err(e) => Err(e),
    }
}

fn wait_for_daemon_start(child: &mut Child) -> std::io::Result<()> {
    let deadline = Instant::now() + Duration::from_secs(2);
    while Instant::now() < deadline {
        if daemon_running()? {
            println!("shellclaw: started");
            return Ok(());
        }
        if let Some(status) = child.try_wait()? {
            return Err(std::io::Error::other(format!(
                "daemon exited before its socket was ready ({status})"
            )));
        }
        std::thread::sleep(Duration::from_millis(25));
    }
    Err(std::io::Error::new(
        std::io::ErrorKind::TimedOut,
        "daemon socket was not ready within 2 seconds",
    ))
}

/// 停止 daemon:寻找并终止运行中的 shellclaw daemon 进程。
fn cmd_stop() -> std::io::Result<()> {
    let config = DaemonConfig::load();
    let pid_from_file =
        std::fs::read_to_string(pid_path()).ok().and_then(|value| value.trim().parse::<u32>().ok());
    let active_sockets = [Some(config.socket_path.clone()), legacy_socket_path()]
        .into_iter()
        .flatten()
        .filter(|path| socket_running(path))
        .collect::<Vec<_>>();

    let instance = pid_from_file
        .filter(|pid| is_shellclaw_daemon(*pid))
        .zip(active_sockets.first().cloned())
        .or_else(|| {
            active_sockets.iter().find_map(|socket| {
                socket_owner_pid(socket)
                    .filter(|pid| is_shellclaw_daemon(*pid))
                    .map(|pid| (pid, socket.clone()))
            })
        });

    match instance {
        Some((pid, active_socket)) => {
            let status = Command::new("kill").arg("-TERM").arg(pid.to_string()).status()?;
            if status.success() {
                let deadline = Instant::now() + Duration::from_secs(2);
                while Instant::now() < deadline && socket_running(&active_socket) {
                    std::thread::sleep(Duration::from_millis(25));
                }
                if socket_running(&active_socket) {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::TimedOut,
                        "daemon did not stop within 2 seconds",
                    ));
                }
                let _ = std::fs::remove_file(&active_socket);
                remove_pid_file_if_matches(pid);
                println!("shellclaw: stopped (pid {pid})");
                Ok(())
            } else {
                Err(std::io::Error::other("failed to stop daemon"))
            }
        }
        None => {
            if pid_from_file.is_some() {
                let _ = std::fs::remove_file(pid_path());
            }
            println!("shellclaw: not running");
            Ok(())
        }
    }
}

fn cmd_status() -> std::io::Result<()> {
    if daemon_running()? {
        println!("shellclaw: running");
    } else {
        println!("shellclaw: not running");
    }
    Ok(())
}

/// 处理 log on/off:持久化到 ~/.shellclaw/config。
fn cmd_log(args: &[String]) -> std::io::Result<()> {
    match args.get(1).map(String::as_str) {
        Some("on") | Some("enable") | Some("1") => {
            config::set_config_bool("log_enabled", true)?;
            println!("shellclaw: file logging enabled (daemon will log on next start)");
            Ok(())
        }
        Some("off") | Some("disable") | Some("0") => {
            config::set_config_bool("log_enabled", false)?;
            println!("shellclaw: file logging disabled");
            Ok(())
        }
        _ => {
            // 查询当前状态
            let on = config::DaemonConfig::load().log_enabled;
            println!("shellclaw: file logging {}", if on { "enabled" } else { "disabled" });
            Ok(())
        }
    }
}

fn cmd_setup(args: &[String]) -> std::io::Result<()> {
    let hook_dir = args.get(1).map(std::path::PathBuf::from).ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "usage: shellclaw setup /path/to/share/shellclaw",
        )
    })?;
    let home = std::env::var_os("HOME")
        .map(std::path::PathBuf::from)
        .ok_or_else(|| std::io::Error::other("HOME is not set"))?;

    install_hook_block(&home.join(".zshrc"), &hook_dir.join("shellclaw.zsh"))?;
    cmd_start(&[])
}

fn install_hook_block(
    rc_path: &std::path::Path,
    hook_path: &std::path::Path,
) -> std::io::Result<()> {
    if !hook_path.is_file() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("hook not found: {}", hook_path.display()),
        ));
    }

    const START: &str = "# >>> shellclaw >>>";
    const END: &str = "# <<< shellclaw <<<";
    let existing = std::fs::read_to_string(rc_path).unwrap_or_default();
    let mut filtered = String::new();
    let mut skipping = false;
    for line in existing.lines() {
        if line == START {
            skipping = true;
            continue;
        }
        if line == END {
            skipping = false;
            continue;
        }
        if !skipping {
            filtered.push_str(line);
            filtered.push('\n');
        }
    }
    while filtered.ends_with("\n\n") {
        filtered.pop();
    }

    let hook = hook_path.to_string_lossy().replace('\'', "'\\''");
    filtered.push_str(&format!("{START}\n[ -r '{hook}' ] && source '{hook}'\n{END}\n"));
    if let Some(parent) = rc_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let temp = rc_path.with_extension("shellclaw.tmp");
    std::fs::write(&temp, filtered)?;
    std::fs::rename(temp, rc_path)
}

/// 检查 daemon 是否在运行(socket 存在即可,近似判断)。
fn daemon_running() -> std::io::Result<bool> {
    let socket_path = DaemonConfig::load().socket_path;
    Ok(socket_running(&socket_path))
}

fn socket_running(socket_path: &std::path::Path) -> bool {
    socket_path.exists() && std::os::unix::net::UnixStream::connect(socket_path).is_ok()
}

fn legacy_socket_path() -> Option<PathBuf> {
    if std::env::var_os("SHELLCLAW_DATA_DIR").is_none()
        && std::env::var_os("SHELLCLAW_SOCKET_PATH").is_none()
    {
        Some(PathBuf::from("/tmp/shellclaw.sock"))
    } else {
        None
    }
}

fn socket_owner_pid(socket_path: &std::path::Path) -> Option<u32> {
    let output = Command::new("lsof").arg("-t").arg(socket_path).output().ok()?;
    if !output.status.success() {
        return None;
    }
    String::from_utf8_lossy(&output.stdout).lines().find_map(|line| line.trim().parse::<u32>().ok())
}

fn pid_path() -> PathBuf {
    config::data_dir().join("daemon.pid")
}

fn process_exists(pid: u32) -> bool {
    Command::new("kill")
        .arg("-0")
        .arg(pid.to_string())
        .status()
        .map(|status| status.success())
        .unwrap_or(false)
}

fn is_shellclaw_daemon(pid: u32) -> bool {
    let output =
        match Command::new("ps").args(["-ww", "-p", &pid.to_string(), "-o", "command="]).output() {
            Ok(output) if output.status.success() => output,
            _ => return false,
        };
    let command = String::from_utf8_lossy(&output.stdout);
    command.contains("shellclaw") && command.split_whitespace().any(|arg| arg == "daemon")
}

fn remove_pid_file_if_matches(pid: u32) {
    let path = pid_path();
    let matches = std::fs::read_to_string(&path)
        .ok()
        .is_some_and(|contents| contents.trim() == pid.to_string());
    if matches {
        let _ = std::fs::remove_file(path);
    }
}

struct DaemonPidFile {
    pid_path: PathBuf,
    pid: u32,
}

impl DaemonPidFile {
    fn create() -> std::io::Result<Self> {
        let pid_path = pid_path();
        if let Some(parent) = pid_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        if let Ok(existing) = std::fs::read_to_string(&pid_path) {
            if let Ok(pid) = existing.trim().parse::<u32>() {
                if process_exists(pid) && is_shellclaw_daemon(pid) {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::AlreadyExists,
                        "ShellClaw daemon is already running",
                    ));
                }
            }
            let _ = std::fs::remove_file(&pid_path);
        }
        let pid = std::process::id();
        use std::io::Write;
        let mut file = std::fs::OpenOptions::new().write(true).create_new(true).open(&pid_path)?;
        file.write_all(pid.to_string().as_bytes())?;
        Ok(Self { pid_path, pid })
    }
}

impl Drop for DaemonPidFile {
    fn drop(&mut self) {
        let matches = std::fs::read_to_string(&self.pid_path)
            .ok()
            .is_some_and(|contents| contents.trim() == self.pid.to_string());
        if matches {
            let _ = std::fs::remove_file(&self.pid_path);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::install_hook_block;

    fn test_root(name: &str) -> std::path::PathBuf {
        std::env::temp_dir().join(format!(
            "shellclaw-{name}-{}-{:?}",
            std::process::id(),
            std::thread::current().id()
        ))
    }

    #[test]
    fn setup_is_idempotent_and_preserves_user_config() {
        let root = test_root("setup");
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).unwrap();
        let rc = root.join(".zshrc");
        let hook = root.join("shellclaw.zsh");
        std::fs::write(&rc, "export USER_SETTING=1\n").unwrap();
        std::fs::write(&hook, "# hook\n").unwrap();

        install_hook_block(&rc, &hook).unwrap();
        install_hook_block(&rc, &hook).unwrap();
        let content = std::fs::read_to_string(&rc).unwrap();
        assert!(content.contains("export USER_SETTING=1"));
        assert_eq!(content.matches("# >>> shellclaw >>>").count(), 1);
        assert_eq!(content.matches("source '").count(), 1);

        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn setup_quotes_hook_path_and_replaces_old_marker_block() {
        let root = test_root("setup-quote");
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).unwrap();
        let rc = root.join(".zshrc");
        let hook_dir = root.join("share's hooks");
        std::fs::create_dir_all(&hook_dir).unwrap();
        let hook = hook_dir.join("shellclaw.zsh");
        std::fs::write(
            &rc,
            "before\n# >>> shellclaw >>>\nsource '/old/hook'\n# <<< shellclaw <<<\nafter\n",
        )
        .unwrap();
        std::fs::write(&hook, "# hook\n").unwrap();

        install_hook_block(&rc, &hook).unwrap();
        let content = std::fs::read_to_string(&rc).unwrap();
        assert!(content.contains("before\nafter\n"));
        assert!(!content.contains("/old/hook"));
        assert!(content.contains("share'\\''s hooks/shellclaw.zsh"));
        assert_eq!(content.matches("# >>> shellclaw >>>").count(), 1);

        let _ = std::fs::remove_dir_all(root);
    }
}
