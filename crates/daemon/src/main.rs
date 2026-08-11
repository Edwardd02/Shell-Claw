mod config;
mod diagnostics;
mod ipc;
mod memory;
mod model;
mod scheduler;

use config::DaemonConfig;
use ipc::server::IpcServer;
use std::process::Command;
use tracing::info;

const SOCKET_PATH: &str = "/tmp/shellclaw.sock";

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();

    match args.first().map(String::as_str) {
        Some("daemon") => run_daemon().await,
        Some("start") => cmd_start(&args),
        Some("stop") => cmd_stop(),
        Some("status") => cmd_status(),
        Some("log") => cmd_log(&args),
        Some("help") | Some("-h") | Some("--help") => { print_usage(); Ok(()) }
        _ => { print_usage(); Ok(()) }
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
    println!("  shellclaw help           显示帮助");
}

/// 前台运行 daemon(供 launchd/systemd 或 shellclaw start 的后台进程调用)。
async fn run_daemon() -> std::io::Result<()> {
    let config = DaemonConfig::load();

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
    // 清理可能残留的 socket
    let _ = std::fs::remove_file(SOCKET_PATH);

    let exe = std::env::current_exe()?;
    let child = Command::new(exe)
        .arg("daemon")
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn();
    match child {
        Ok(_) => {
            println!("shellclaw: started");
            Ok(())
        }
        Err(e) => Err(e),
    }
}

/// 停止 daemon:寻找并终止运行中的 shellclaw daemon 进程。
fn cmd_stop() -> std::io::Result<()> {
    let pid = find_daemon_pid();
    match pid {
        Some(pid) => {
            // Unix 直接 kill TERM
            unsafe {
                libc_kill(pid);
            }
            println!("shellclaw: stopped (pid {pid})");
            let _ = std::fs::remove_file(SOCKET_PATH);
            Ok(())
        }
        None => {
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

/// 检查 daemon 是否在运行(socket 存在即可,近似判断)。
fn daemon_running() -> std::io::Result<bool> {
    Ok(std::path::Path::new(SOCKET_PATH).exists())
}

/// 找到运行中的 daemon pid(通过 pgrep 匹配二进制名)。
fn find_daemon_pid() -> Option<i32> {
    let exe = std::env::current_exe().ok()?;
    let name = exe.file_name()?.to_string_lossy().to_string();
    let out = Command::new("pgrep")
        .arg("-f")
        .arg(&name)
        .output()
        .ok()?;
    let out = String::from_utf8_lossy(&out.stdout);
    out.lines()
        .filter_map(|l| l.trim().parse::<i32>().ok())
        .find(|&p| p != std::process::id() as i32)
}

#[cfg(unix)]
fn libc_kill(pid: i32) {
    unsafe {
        libc::kill(pid, libc::SIGTERM);
    }
}
