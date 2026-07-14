//! W3 Link frame contract tests.

use pulse_link_core::{
    LinkFrameError, LinkFrameHeader, LinkMessageKind, FRAME_HEADER_BYTES,
    MAX_FULL_SNAPSHOT_PAYLOAD_BYTES, MAX_HOOK_INGRESS_PAYLOAD_BYTES, MAX_ISLAND_PAYLOAD_BYTES,
};

#[test]
fn link_frame_header_round_trips_without_payload_content() -> Result<(), String> {
    let header = LinkFrameHeader {
        message_kind: LinkMessageKind::HookEnvelope,
        request_id: 42,
        payload_length: 128,
    };

    let encoded = header.encode();
    assert_eq!(encoded.len(), FRAME_HEADER_BYTES);

    let decoded = LinkFrameHeader::decode(&encoded).map_err(|error| format!("{error:?}"))?;

    assert_eq!(decoded, header);
    Ok(())
}

#[test]
fn link_frame_rejects_bad_magic_and_oversized_payload_before_payload_parse() -> Result<(), String> {
    let mut encoded = LinkFrameHeader {
        message_kind: LinkMessageKind::HookEnvelope,
        request_id: 1,
        payload_length: MAX_HOOK_INGRESS_PAYLOAD_BYTES as u32 + 1,
    }
    .encode();

    assert_eq!(
        LinkFrameHeader::decode(&encoded),
        Err(LinkFrameError::PayloadTooLarge)
    );

    encoded[0] = b'X';
    assert_eq!(
        LinkFrameHeader::decode(&encoded),
        Err(LinkFrameError::BadMagic)
    );
    Ok(())
}

#[test]
fn link_message_kinds_have_spike_c_payload_caps() {
    assert_eq!(
        LinkMessageKind::HookEnvelope.max_payload_bytes(),
        MAX_HOOK_INGRESS_PAYLOAD_BYTES
    );
    assert_eq!(
        LinkMessageKind::IslandControl.max_payload_bytes(),
        MAX_ISLAND_PAYLOAD_BYTES
    );
    assert_eq!(
        LinkMessageKind::FullSnapshot.max_payload_bytes(),
        MAX_FULL_SNAPSHOT_PAYLOAD_BYTES
    );
}
