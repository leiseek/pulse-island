//! W3 initial handoff contract tests.

use pulse_link_core::{
    InitialHandoffPlan, LinkFrameError, LinkFrameHeader, LinkMessageKind,
    MAX_HOOK_INGRESS_PAYLOAD_BYTES,
};

#[test]
fn initial_handoff_uses_inherited_handle_not_command_line_environment_or_temp_file(
) -> Result<(), LinkFrameError> {
    let header = LinkFrameHeader {
        message_kind: LinkMessageKind::HookEnvelope,
        request_id: 7,
        payload_length: 64,
    };

    let plan = InitialHandoffPlan::new(header)?;

    assert_eq!(plan.argv, ["--wake-if-needed", "--handoff-stdin"]);
    assert!(plan.inherited_handoff_stdin);
    assert!(plan.environment.is_empty());
    assert!(plan.temp_file_name.is_none());
    assert_eq!(plan.frame_header, header);
    Ok(())
}

#[test]
fn initial_handoff_rejects_oversized_hook_payload_before_launch_plan() {
    let header = LinkFrameHeader {
        message_kind: LinkMessageKind::HookEnvelope,
        request_id: 7,
        payload_length: MAX_HOOK_INGRESS_PAYLOAD_BYTES as u32 + 1,
    };

    assert_eq!(
        InitialHandoffPlan::new(header),
        Err(LinkFrameError::PayloadTooLarge)
    );
}
