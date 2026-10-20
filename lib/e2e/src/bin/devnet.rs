//! Runs the Stylus devnet used by the end-to-end tests and benches.
//!
//! `devnet up` starts a nitro dev node with `stylus_tools::devnet::Node`
//! (the same node the Stylus SDK uses for its integration tests), writes its
//! RPC URL to `target/devnet/rpc` and keeps the node alive until it receives
//! SIGINT or SIGTERM. `devnet down` stops a running daemon.
use std::{fs, process::Command, time::Duration};

use e2e::{daemon_pid, devnet_dir, pid_file, rpc_file};
use stylus_tools::devnet::Node;
use tokio::signal::unix::{signal, SignalKind};

#[tokio::main]
async fn main() -> eyre::Result<()> {
    match std::env::args().nth(1).as_deref() {
        Some("up") => up().await,
        Some("down") => down(),
        Some("status") => status(),
        _ => {
            eprintln!("usage: devnet up | down | status");
            std::process::exit(2);
        }
    }
}

async fn up() -> eyre::Result<()> {
    if let Ok(pid) = daemon_pid() {
        if alive(pid) {
            eyre::bail!(
                "a devnet daemon is already running (pid {pid}, rpc {})",
                fs::read_to_string(rpc_file()).unwrap_or_default().trim()
            );
        }
    }
    let dir = devnet_dir();
    fs::create_dir_all(&dir)?;
    let _ = fs::remove_file(rpc_file());

    eprintln!("starting the nitro dev node...");
    let node = Node::new().await?;
    fs::write(pid_file(), std::process::id().to_string())?;
    let tmp = dir.join("rpc.tmp");
    fs::write(&tmp, node.rpc())?;
    fs::rename(tmp, rpc_file())?;
    println!("{}", node.rpc());

    let mut terminate = signal(SignalKind::terminate())?;
    let mut interrupt = signal(SignalKind::interrupt())?;
    tokio::select! {
        _ = terminate.recv() => {}
        _ = interrupt.recv() => {}
    }
    eprintln!("stopping the nitro dev node...");
    let _ = fs::remove_file(rpc_file());
    let _ = fs::remove_file(pid_file());
    drop(node);
    Ok(())
}

fn down() -> eyre::Result<()> {
    let Ok(pid) = daemon_pid() else {
        println!("no devnet daemon is running");
        return Ok(());
    };
    if !alive(pid) {
        let _ = fs::remove_file(rpc_file());
        let _ = fs::remove_file(pid_file());
        println!("no devnet daemon is running");
        return Ok(());
    }
    Command::new("kill").arg(pid.to_string()).status()?;
    for _ in 0..600 {
        if !alive(pid) {
            println!("devnet stopped");
            return Ok(());
        }
        std::thread::sleep(Duration::from_millis(100));
    }
    eyre::bail!("the devnet daemon (pid {pid}) did not stop in time")
}

fn status() -> eyre::Result<()> {
    match daemon_pid() {
        Ok(pid) if alive(pid) => {
            println!("running (pid {pid}): {}", e2e::rpc_url()?);
        }
        _ => println!("not running"),
    }
    Ok(())
}

fn alive(pid: u32) -> bool {
    Command::new("kill")
        .args(["-0", &pid.to_string()])
        .status()
        .is_ok_and(|status| status.success())
}
