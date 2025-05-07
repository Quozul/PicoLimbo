use std::process::Stdio;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, ChildStdin, Command};
use tokio::sync::Mutex;
use tokio_util::sync::CancellationToken;
use tracing::{debug, error, info};

pub struct ServerManager {
    start_command: String,
    server: Arc<Mutex<Option<ServerProcess>>>,
    done: Arc<AtomicBool>,
    stop_token: Arc<Mutex<Option<CancellationToken>>>,
    input_listener_token: CancellationToken,
}

struct ServerProcess {
    child: Child,
    // Wrap the child's stdin so that it can be shared safely.
    stdin: Arc<Mutex<ChildStdin>>,
}

impl ServerProcess {
    pub fn new(child: Child, stdin: ChildStdin) -> Self {
        Self {
            child,
            stdin: Arc::new(Mutex::new(stdin)),
        }
    }
}

impl ServerManager {
    const DONE_MESSAGE: &'static str = ")! For help, type ";
    const STOP_COMMAND: &'static [u8; 5] = b"stop\n";

    /// Creates a new ServerManager and spawns a global input listener.
    pub fn new(start_command: String) -> Self {
        let manager = Self {
            start_command,
            server: Arc::new(Mutex::new(None)),
            done: Arc::new(AtomicBool::new(false)),
            stop_token: Arc::new(Mutex::new(None)),
            input_listener_token: CancellationToken::new(),
        };

        // Always listen to parent's stdin.
        manager.spawn_input_listener();
        manager
    }

    /// Spawns a task that continuously reads from the parent's stdin.
    fn spawn_input_listener(&self) {
        let server_clone = self.server.clone();
        let cancellation_token = self.input_listener_token.clone();

        tokio::spawn(async move {
            let stdin = tokio::io::stdin();
            let mut reader = BufReader::new(stdin);
            let mut line = String::new();

            loop {
                tokio::select! {
                    _ = cancellation_token.cancelled() => {
                        debug!("Input listener cancelled");
                        break;
                    }
                    result = reader.read_line(&mut line) => {
                        match result {
                            Ok(0) => break, // EOF reached.
                            Ok(_) => {
                                let maybe_proc = server_clone.lock().await;
                                if let Some(proc) = maybe_proc.as_ref() {
                                    let mut child_stdin = proc.stdin.lock().await;
                                    if let Err(e) = child_stdin.write_all(line.as_bytes()).await {
                                        error!("Failed to send input to child process: {}", e);
                                    }
                                    if let Err(e) = child_stdin.flush().await {
                                        error!("Failed to flush child's stdin: {}", e);
                                    }
                                } else {
                                    error!("Server is not running");
                                }
                            }
                            Err(e) => {
                                error!("Error reading from parent's stdin: {}", e);
                                break;
                            }
                        }
                    }
                }
            }
        });
    }

    pub fn stop_stdin_listener(&self) {
        self.input_listener_token.cancel();
    }

    pub async fn start_server(&self) -> anyhow::Result<()> {
        let mut server_lock = self.server.lock().await;
        if server_lock.is_some() {
            anyhow::bail!("Server is already running");
        }

        let mut parts = self.start_command.split_whitespace();
        let program = parts
            .next()
            .ok_or_else(|| anyhow::anyhow!("No command provided to start the server"))?;
        let args: Vec<&str> = parts.collect();

        let mut child = Command::new(program)
            .args(&args)
            .stdout(Stdio::piped())
            .stdin(Stdio::piped())
            .spawn()?;

        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| anyhow::anyhow!("Failed to capture stdout"))?;
        let stdin = child
            .stdin
            .take()
            .ok_or_else(|| anyhow::anyhow!("Failed to capture stdin"))?;

        *server_lock = Some(ServerProcess::new(child, stdin));
        drop(server_lock);

        // Spawn the output monitor.
        self.spawn_output_monitor(BufReader::new(stdout));

        Ok(())
    }

    fn spawn_output_monitor(
        &self,
        reader: BufReader<impl tokio::io::AsyncRead + Unpin + Send + 'static>,
    ) {
        let done_flag = self.done.clone();
        let server = self.server.clone();
        tokio::spawn(async move {
            let mut lines = reader.lines();
            while let Ok(Some(line)) = lines.next_line().await {
                println!("{line}");
                if line.contains(Self::DONE_MESSAGE) {
                    done_flag.store(true, Ordering::SeqCst);
                }
            }
            // When output ends, wait for the process to exit.
            if let Some(mut proc) = server.lock().await.take() {
                if let Err(e) = proc.child.wait().await {
                    error!("Error waiting for process: {:?}", e);
                }
                info!("Server has stopped");
            }
            done_flag.store(false, Ordering::SeqCst);
        });
    }

    /// Schedules a stop only if no stop task is already pending.
    pub async fn schedule_stop(&self, delay: Duration) {
        let mut token_lock = self.stop_token.lock().await;
        if token_lock.is_some() {
            debug!("A stop task is already scheduled; not scheduling another.");
            return;
        }

        let server = self.server.clone();
        if delay.is_zero() {
            info!("Stopping server now…");
            if let Err(e) = Self::stop_now(server).await {
                error!("Failed to stop server: {}", e);
            }
            return;
        }

        let token = CancellationToken::new();
        *token_lock = Some(token.clone());
        info!("Server scheduled to stop in {} seconds", delay.as_secs());
        tokio::spawn(async move {
            tokio::select! {
                _ = tokio::time::sleep(delay) => {
                    if let Err(e) = Self::stop_now(server).await {
                        error!("Failed to stop server: {}", e);
                    }
                }
                _ = token.cancelled() => {
                    info!("Scheduled stop cancelled");
                }
            }
        });
    }

    pub async fn cancel_stop(&self) {
        let mut token_lock = self.stop_token.lock().await;
        if let Some(token) = token_lock.take() {
            token.cancel();
        }
    }

    async fn stop_now(server: Arc<Mutex<Option<ServerProcess>>>) -> anyhow::Result<()> {
        let mut lock = server.lock().await;
        if let Some(ref mut proc) = *lock {
            info!("Sending stop command to server");
            let mut stdin = proc.stdin.lock().await;
            stdin.write_all(Self::STOP_COMMAND).await?;
            stdin.flush().await?;
            Ok(())
        } else {
            anyhow::bail!("Server is not running");
        }
    }

    pub async fn get_server_status(&self) -> ServerStatus {
        let server_running = self.server.lock().await.is_some();
        if !server_running {
            ServerStatus::Offline
        } else if self.done.load(Ordering::SeqCst) {
            ServerStatus::Online
        } else {
            ServerStatus::Starting
        }
    }
}

#[derive(Eq, PartialEq, Debug)]
pub enum ServerStatus {
    Offline,
    Starting,
    Online,
}
