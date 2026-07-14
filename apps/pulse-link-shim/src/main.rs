//! Pulse Link shim executable.

#[cfg(target_env = "msvc")]
use std::io::Write;
use std::io::{self, Read};
#[cfg(target_env = "msvc")]
use std::process::{Command, Stdio};

#[cfg(target_env = "msvc")]
use pulse_link_core::{LinkFrameHeader, LinkMessageKind};
#[cfg(target_env = "msvc")]
use pulse_link_shim::ExistingLinkIngressDelivery;
use pulse_link_shim::{
    run_shim_preflight, ShimDelivery, ShimDeliveryAttempt, SHIM_INPUT_LIMIT_BYTES,
};
use pulse_protocol::RejectionCategory;
#[cfg(target_env = "msvc")]
use pulse_protocol::ShimExitStatus;
#[cfg(target_env = "msvc")]
use pulse_win32::LinkLocalObjectNames;

fn main() {
    let diagnostic = std::env::args().any(|argument| argument == "--diagnostic");
    let scope = scoped_diagnostic_argument();
    let safe_mode_enabled = matches!(
        std::env::var("PULSE_SAFE_MODE").as_deref(),
        Ok("1") | Ok("true")
    );
    let (input, read_failed) = read_bounded_stdin();
    #[cfg(target_env = "msvc")]
    let report = if let Some(scope) = scope {
        let names = diagnostic_names(&scope);
        let existing = if provider_argument().as_deref() == Some("codex_cli") {
            if safe_mode_enabled || read_failed {
                pulse_link_shim::ShimRunReport {
                    exit_status: ShimExitStatus::Success,
                    forwarded: false,
                    rejection: None,
                }
            } else {
                match pulse_link_shim::codex_hook_frame(&input, pulse_domain::TimestampMs(0), 1) {
                    Ok((header, payload)) => {
                        let delivered = pulse_win32_link::send_ingress_message_and_wait_ack(
                            &names, &header, &payload,
                        )
                        .is_ok();
                        let delivered = delivered
                            || HandoffWakeDelivery::new(scope.clone())
                                .deliver_frame(&header, &payload);
                        pulse_link_shim::ShimRunReport {
                            exit_status: ShimExitStatus::Success,
                            forwarded: delivered,
                            rejection: None,
                        }
                    }
                    Err(rejection) => pulse_link_shim::ShimRunReport {
                        exit_status: ShimExitStatus::Success,
                        forwarded: false,
                        rejection: Some(rejection),
                    },
                }
            }
        } else {
            let mut delivery = ExistingLinkIngressDelivery::new(names);
            run_shim_preflight(&input, safe_mode_enabled || read_failed, &mut delivery)
        };
        if existing.forwarded || existing.rejection.is_some() || safe_mode_enabled || read_failed {
            existing
        } else {
            let mut wake = HandoffWakeDelivery::new(scope);
            run_shim_preflight(&input, false, &mut wake)
        }
    } else {
        let mut delivery = UnavailableDelivery;
        run_shim_preflight(&input, safe_mode_enabled || read_failed, &mut delivery)
    };

    #[cfg(not(target_env = "msvc"))]
    let report = {
        let _ = scope;
        if provider_argument().as_deref() == Some("codex_cli") && !safe_mode_enabled && !read_failed
        {
            match pulse_link_shim::sanitize_codex_hook(&input, pulse_domain::TimestampMs(0)) {
                Ok(_) => pulse_link_shim::ShimRunReport {
                    exit_status: pulse_protocol::ShimExitStatus::Success,
                    forwarded: false,
                    rejection: None,
                },
                Err(rejection) => pulse_link_shim::ShimRunReport {
                    exit_status: pulse_protocol::ShimExitStatus::Success,
                    forwarded: false,
                    rejection: Some(rejection),
                },
            }
        } else {
            let mut delivery = UnavailableDelivery;
            run_shim_preflight(&input, safe_mode_enabled || read_failed, &mut delivery)
        }
    };

    if diagnostic {
        println!(
            "shim_exit={} forwarded={} rejection={} read_failed={}",
            exit_label(report.exit_status),
            report.forwarded,
            rejection_label(report.rejection),
            read_failed
        );
    }
}

fn scoped_diagnostic_argument() -> Option<String> {
    let mut arguments = std::env::args();
    while let Some(argument) = arguments.next() {
        if argument == "--scope" {
            return arguments.next().filter(|value| !value.is_empty());
        }
    }
    None
}

fn provider_argument() -> Option<String> {
    let mut arguments = std::env::args();
    while let Some(argument) = arguments.next() {
        if argument == "--provider" {
            return arguments.next().filter(|value| !value.is_empty());
        }
    }
    None
}

#[cfg(target_env = "msvc")]
fn diagnostic_names(scope: &str) -> LinkLocalObjectNames {
    LinkLocalObjectNames::derive(scope, "diagnostic-user", "diagnostic-session", 1)
}

#[cfg(target_env = "msvc")]
struct HandoffWakeDelivery {
    scope: String,
}

#[cfg(target_env = "msvc")]
impl HandoffWakeDelivery {
    fn new(scope: String) -> Self {
        Self { scope }
    }

    fn link_executable() -> Option<std::path::PathBuf> {
        let current = std::env::current_exe().ok()?;
        Some(current.with_file_name("pulse-link.exe"))
    }

    fn deliver_frame(&mut self, header: &LinkFrameHeader, payload: &[u8]) -> bool {
        let Some(link_executable) = Self::link_executable() else {
            return false;
        };
        let Ok(mut child) = Command::new(link_executable)
            .args(["--handoff-stdin", "--persistent", "--scope", &self.scope])
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
        else {
            return false;
        };
        let Some(mut stdin) = child.stdin.take() else {
            return false;
        };
        stdin.write_all(&header.encode()).is_ok() && stdin.write_all(payload).is_ok()
    }
}

#[cfg(target_env = "msvc")]
impl ShimDelivery for HandoffWakeDelivery {
    fn deliver(&mut self, _attempt: ShimDeliveryAttempt) -> bool {
        let header = LinkFrameHeader {
            message_kind: LinkMessageKind::HookEnvelope,
            request_id: 1,
            payload_length: 0,
        };
        self.deliver_frame(&header, &[])
    }
}

fn read_bounded_stdin() -> (Vec<u8>, bool) {
    let mut input = Vec::with_capacity(SHIM_INPUT_LIMIT_BYTES.saturating_add(1));
    let mut stdin = io::stdin().lock();
    let result = stdin
        .by_ref()
        .take((SHIM_INPUT_LIMIT_BYTES.saturating_add(1)) as u64)
        .read_to_end(&mut input);
    (input, result.is_err())
}

fn exit_label(status: pulse_protocol::ShimExitStatus) -> &'static str {
    match status {
        pulse_protocol::ShimExitStatus::Success => "success",
    }
}

fn rejection_label(rejection: Option<RejectionCategory>) -> &'static str {
    match rejection {
        None => "none",
        Some(RejectionCategory::Oversized) => "oversized",
        Some(RejectionCategory::UnsupportedVersion) => "unsupported_version",
        Some(RejectionCategory::ForbiddenField) => "forbidden_field",
        Some(RejectionCategory::Malformed) => "malformed",
        Some(RejectionCategory::UnsupportedStructuredSource) => "unsupported_structured_source",
        Some(RejectionCategory::SnapshotTooLarge) => "snapshot_too_large",
    }
}

struct UnavailableDelivery;

impl ShimDelivery for UnavailableDelivery {
    fn deliver(&mut self, _attempt: ShimDeliveryAttempt) -> bool {
        false
    }
}
