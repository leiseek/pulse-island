//! W3 fake Island protocol contract tests.

use pulse_link_core::{FakeIslandSession, IslandControlRequest, IslandDelivery};
use pulse_protocol::{IslandMessage, LinkHealthStatus, PROTOCOL_VERSION};

#[test]
fn fake_island_hello_snapshot_and_subscribe_are_state_only_messages() {
    let mut session = FakeIslandSession::new(7);

    assert_eq!(
        session.handle(IslandControlRequest::Hello),
        IslandDelivery::Message(IslandMessage::HelloAck {
            version: PROTOCOL_VERSION
        })
    );

    let snapshot = session.handle(IslandControlRequest::GetSnapshot);
    match snapshot {
        IslandDelivery::Message(IslandMessage::FullSnapshot(snapshot)) => {
            assert_eq!(snapshot.revision, 7);
            assert!(snapshot.tasks.is_empty());
        }
        other => {
            assert_eq!(
                other,
                IslandDelivery::Message(IslandMessage::ProtocolError(
                    pulse_protocol::ProtocolErrorCategory::Malformed
                ))
            );
        }
    }

    assert_eq!(
        session.handle(IslandControlRequest::Subscribe),
        IslandDelivery::Message(IslandMessage::LinkHealth {
            status: LinkHealthStatus::Healthy
        })
    );
}

#[test]
fn subscribed_island_receives_monotonic_deltas_and_gap_requires_full_snapshot() {
    let mut session = FakeIslandSession::new(10);

    assert_eq!(
        session.publish_delta(11),
        IslandDelivery::NotSubscribed { revision: 11 }
    );

    session.handle(IslandControlRequest::Subscribe);

    let first_delta = session.publish_delta(11);
    match first_delta {
        IslandDelivery::Message(IslandMessage::SnapshotDelta(delta)) => {
            assert_eq!(delta.revision, 11);
            assert!(delta.upserts.is_empty());
            assert!(delta.removals.is_empty());
        }
        other => assert_eq!(
            other,
            IslandDelivery::NeedsFullSnapshot {
                expected_revision: 11,
                received_revision: 11,
            }
        ),
    }

    assert_eq!(
        session.publish_delta(13),
        IslandDelivery::NeedsFullSnapshot {
            expected_revision: 12,
            received_revision: 13,
        }
    );
}
