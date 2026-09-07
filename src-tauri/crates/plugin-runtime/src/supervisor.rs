use std::collections::HashMap;
use std::panic::{self, AssertUnwindSafe};
use std::sync::mpsc::{self, RecvTimeoutError};
use std::sync::Mutex;
use std::thread;
use std::time::{Duration, Instant};

use plugin_sdk::{
    ErrorCode, HealthStatus, PluginError, PluginId, PluginResult, ResourceBudget,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum BreakerState {
    Closed,
    Open { remaining_ms: u64 },
    HalfOpen,
}

#[derive(Debug, Clone)]
pub struct CircuitBreaker {
    policy: BreakerPolicy,
    consecutive_failures: u32,
    opened_at_ms: Option<u64>,
    probing: bool,
}

#[derive(Debug, Clone, Copy)]
pub struct BreakerPolicy {
    pub max_consecutive_failures: u32,
    pub cooldown_ms: u64,
}

impl Default for BreakerPolicy {
    fn default() -> Self {
        Self {
            max_consecutive_failures: 3,
            cooldown_ms: 30_000,
        }
    }
}

impl From<&ResourceBudget> for BreakerPolicy {
    fn from(budget: &ResourceBudget) -> Self {
        Self {
            max_consecutive_failures: budget.max_consecutive_failures,
            cooldown_ms: budget.cooldown_ms,
        }
    }
}

impl CircuitBreaker {
    pub fn new(policy: BreakerPolicy) -> Self {
        Self {
            policy,
            consecutive_failures: 0,
            opened_at_ms: None,
            probing: false,
        }
    }

    pub fn state(&self) -> BreakerState {
        match self.opened_at_ms {
            None => BreakerState::Closed,
            Some(opened_at) => {
                let elapsed = now_ms().saturating_sub(opened_at);
                if elapsed >= self.policy.cooldown_ms {
                    BreakerState::HalfOpen
                } else {
                    BreakerState::Open {
                        remaining_ms: self.policy.cooldown_ms - elapsed,
                    }
                }
            }
        }
    }

    pub fn allow(&mut self) -> bool {
        match self.state() {
            BreakerState::Closed => true,
            BreakerState::Open { .. } => false,
            BreakerState::HalfOpen => {
                if self.probing {
                    false // 探测已在进行中，不再放行
                } else {
                    self.probing = true;
                    true
                }
            }
        }
    }

    pub fn record_success(&mut self) {
        self.consecutive_failures = 0;
        self.opened_at_ms = None;
        self.probing = false;
    }

    pub fn record_failure(&mut self) -> bool {
        self.probing = false;
        self.consecutive_failures = self.consecutive_failures.saturating_add(1);
        if self.consecutive_failures >= self.policy.max_consecutive_failures
            && self.opened_at_ms.is_none()
        {
            self.opened_at_ms = Some(now_ms());
            return true;
        }
        false
    }

    pub fn consecutive_failures(&self) -> u32 {
        self.consecutive_failures
    }

    pub fn reset(&mut self) {
        self.consecutive_failures = 0;
        self.opened_at_ms = None;
        self.probing = false;
    }
}

#[derive(Debug, Clone)]
pub struct CallReport {
    pub plugin: PluginId,
    pub elapsed: Duration,
    pub outcome: CallOutcome,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CallOutcome {
    Ok,
    Err,
    Panic,
    Timeout,
    Rejected,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CascadePolicy {
    IsolateOnly,
    StopDependents,
}

pub struct Supervisor {
    breakers: Mutex<HashMap<PluginId, CircuitBreaker>>,
    default_policy: BreakerPolicy,
    cascade: CascadePolicy,
    recent: Mutex<std::collections::VecDeque<CallReport>>,
    max_recent: usize,
}

impl Supervisor {
    pub fn new(cascade: CascadePolicy) -> Self {
        Self {
            breakers: Mutex::new(HashMap::new()),
            default_policy: BreakerPolicy::default(),
            cascade,
            recent: Mutex::new(std::collections::VecDeque::with_capacity(64)),
            max_recent: 64,
        }
    }

    pub fn cascade_policy(&self) -> CascadePolicy {
        self.cascade
    }

    pub fn register(&self, plugin: &PluginId, budget: &ResourceBudget) {
        if let Ok(mut lock) = self.breakers.lock() {
            lock.insert(plugin.clone(), CircuitBreaker::new(BreakerPolicy::from(budget)));
        }
    }

    pub fn unregister(&self, plugin: &PluginId) {
        if let Ok(mut lock) = self.breakers.lock() {
            lock.remove(plugin);
        }
    }

    pub fn state_of(&self, plugin: &PluginId) -> BreakerState {
        self.breakers
            .lock()
            .ok()
            .and_then(|lock| lock.get(plugin).map(|b| b.state()))
            .unwrap_or(BreakerState::Closed)
    }

    pub fn should_cascade(&self) -> bool {
        self.cascade == CascadePolicy::StopDependents
    }

    pub fn reset(&self, plugin: &PluginId) {
        if let Ok(mut lock) = self.breakers.lock() {
            if let Some(breaker) = lock.get_mut(plugin) {
                breaker.reset();
            }
        }
    }

    pub fn recent_calls(&self) -> Vec<CallReport> {
        self.recent
            .lock()
            .ok()
            .map(|lock| lock.iter().cloned().collect())
            .unwrap_or_default()
    }

    fn push_report(&self, report: CallReport) {
        if let Ok(mut lock) = self.recent.lock() {
            if lock.len() >= self.max_recent {
                lock.pop_front();
            }
            lock.push_back(report.clone());
        }
    }

    pub fn guard<T, F>(&self, plugin: &PluginId, timeout: Duration, f: F) -> PluginResult<T>
    where
        T: Send + 'static,
        F: FnOnce() -> PluginResult<T> + Send + 'static,
    {
        if !self.acquire(plugin) {
            self.push_report(CallReport {
                plugin: plugin.clone(),
                elapsed: Duration::ZERO,
                outcome: CallOutcome::Rejected,
            });
            return Err(PluginError::new(
                ErrorCode::ServiceUnavailable,
                format!("plugin '{plugin}' is circuit-broken"),
            ));
        }

        let started = Instant::now();
        let (sender, receiver) = mpsc::channel();
        let worker = thread::Builder::new()
            .name(format!("plugin-call-{}", plugin.as_str()))
            .spawn(move || {
                // catch_unwind 只捕获 unwind；abort 类 panic 仍然会终止进程，
                // 这一点无法在宿主侧兜住，只能靠插件自律。
                let outcome = panic::catch_unwind(AssertUnwindSafe(f));
                let _ = sender.send(outcome);
            });

        let worker = match worker {
            Ok(handle) => handle,
            Err(error) => {
                self.record_failure(plugin);
                return Err(PluginError::io(format!(
                    "cannot spawn worker for plugin '{plugin}': {error}"
                )));
            }
        };

        let received = receiver.recv_timeout(timeout);
        let elapsed = started.elapsed();

        match received {
            Ok(Ok(Ok(value))) => {
                self.record_success(plugin);
                self.push_report(CallReport {
                    plugin: plugin.clone(),
                    elapsed,
                    outcome: CallOutcome::Ok,
                });
                Ok(value)
            }
            Ok(Ok(Err(error))) => {
                self.record_failure(plugin);
                self.push_report(CallReport {
                    plugin: plugin.clone(),
                    elapsed,
                    outcome: CallOutcome::Err,
                });
                Err(error.with_plugin(plugin.clone()))
            }
            Ok(Err(panic_payload)) => {
                self.record_failure(plugin);
                self.push_report(CallReport {
                    plugin: plugin.clone(),
                    elapsed,
                    outcome: CallOutcome::Panic,
                });
                Err(PluginError::new(
                    ErrorCode::Panic,
                    format!("plugin '{plugin}' panicked: {}", panic_message(&panic_payload)),
                )
                .with_plugin(plugin.clone()))
            }
            Err(RecvTimeoutError::Timeout | RecvTimeoutError::Disconnected) => {
                // 超时或线程已退出但未发消息：一律按超时处理，并 detach 线程。
                self.record_failure(plugin);
                self.push_report(CallReport {
                    plugin: plugin.clone(),
                    elapsed,
                    outcome: CallOutcome::Timeout,
                });
                let _ = worker.join_timeout_ignore();
                Err(PluginError::timeout(format!(
                    "plugin '{plugin}' exceeded its {:?} call budget",
                    timeout
                ))
                .with_plugin(plugin.clone()))
            }
        }
    }

    fn acquire(&self, plugin: &PluginId) -> bool {
        let mut lock = match self.breakers.lock() {
            Ok(lock) => lock,
            Err(_) => return true,
        };
        match lock.get_mut(plugin) {
            Some(breaker) => breaker.allow(),
            None => {
                lock.insert(plugin.clone(), CircuitBreaker::new(self.default_policy));
                true
            }
        }
    }

    fn record_success(&self, plugin: &PluginId) {
        if let Ok(mut lock) = self.breakers.lock() {
            lock.entry(plugin.clone())
                .or_insert_with(|| CircuitBreaker::new(self.default_policy))
                .record_success();
        }
    }

    pub fn record_failure(&self, plugin: &PluginId) -> bool {
        self.breakers
            .lock()
            .ok()
            .map(|mut lock| {
                lock.entry(plugin.clone())
                    .or_insert_with(|| CircuitBreaker::new(self.default_policy))
                    .record_failure()
            })
            .unwrap_or(false)
    }
}

fn panic_message(payload: &Box<dyn std::any::Any + Send>) -> String {
    if let Some(text) = payload.downcast_ref::<&str>() {
        (*text).to_string()
    } else if let Some(text) = payload.downcast_ref::<String>() {
        text.clone()
    } else {
        "<non-string panic payload>".to_string()
    }
}

trait JoinTimeoutIgnore {
    fn join_timeout_ignore(self) -> bool;
}

impl JoinTimeoutIgnore for thread::JoinHandle<()> {
    fn join_timeout_ignore(self) -> bool {
        // 一次极短的阻塞等待兜底：真正在跑的线程不会被拖住太久。
        let (sender, receiver) = mpsc::channel();
        let _ = thread::Builder::new()
            .name("plugin-joiner".into())
            .spawn(move || {
                let _ = self.join();
                let _ = sender.send(());
            });
        receiver.recv_timeout(Duration::from_millis(0)).is_ok()
    }
}

pub fn combined_health(reported: HealthStatus, breaker: BreakerState) -> HealthStatus {
    match breaker {
        BreakerState::Open { .. } => HealthStatus::Unhealthy,
        BreakerState::HalfOpen => HealthStatus::Degraded,
        BreakerState::Closed => reported,
    }
}

pub(crate) fn now_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}