# Implementation Plan: Two-Player Online Co-op

**Date**: 2026-11-13 | **Spec**: First-pass requirements derived from the user request and confirmed decisions

## Summary

Add two-player online co-op while retaining the existing offline run. One player hosts and runs the authoritative simulation; the other connects by entering the host's IP address and UDP port. The host owns gameplay outcomes, and clients send bounded player inputs rather than authoritative combat or entity changes.

The initial release is limited to two players, direct IP, and a listen-server host. It does not include matchmaking, relay/NAT traversal, dedicated servers, host migration, or joining an active run.

## Technical Context

**Language/Version**: Rust 2024; Bevy 0.19.1  
**Primary Dependencies**: `bevy_renet` 5.0.0 with its Netcode transport; `serde` and a bounded message serializer for the versioned protocol. The crate's published compatibility table maps Bevy 0.19 to `bevy_renet` 5.0 ([crate documentation](https://crates.io/crates/bevy_renet/5.0.0)).  
**Storage**: None; session state is in memory.  
**Testing**: Existing `cargo test`, protocol/unit tests, and host-client loopback integration tests.  
**Target Platform**: Existing native desktop targets; validate on Windows first.  
**Performance Goals**: 30 Hz authoritative network snapshots as a starting point; keep local input responsive with local-player prediction and interpolate remote entities. Tune snapshot rate and payload size with measurements.  
**Constraints**: Direct-IP UDP requires a reachable host address and often router port forwarding. No relay or NAT traversal is included. Keep `GameSet` ordering and host-only execution explicit; use stable network IDs rather than serializing Bevy `Entity` values.

## Constitution Check

No project constitution was found. No known repository constraint conflicts with this plan. Preserve the existing offline mode and avoid introducing a separate server executable for the first release. Recheck if the project adds a constitution.

## Applied Guidelines

No matching framework-migration guidelines apply. Networking choices are based on the published `bevy_renet` 5.0.0 documentation and its Bevy 0.19 compatibility entry.

## Implementation Steps

### Phase 1: Foundational networking

#### Step 1.1: [Cross-cutting] Add transport and protocol foundations
- **Requirements**: REQ-001, REQ-002, REQ-007
- **Description**: Add the Bevy 0.19-compatible Renet integration and define a versioned, size-bounded protocol for connection handshakes, player inputs, acknowledgements, snapshots, and entity lifecycle events. Keep transport setup separate from gameplay systems.

#### Step 1.2: [Cross-cutting] Add network roles and stable identities
- **Requirements**: REQ-001, REQ-003, REQ-004
- **Description**: Introduce explicit offline/host/client session roles, peer IDs, player slots, and stable network entity IDs. Extend game-state transitions for connecting and lobby states without changing offline startup behavior.

**Tasks**

- [ ] T001 [Plan:1.1] Add `bevy_renet` 5.0.0 with Netcode transport and serialization dependencies in `Cargo.toml`; define bounded, versioned messages in `src/network/protocol.rs`.
- [ ] T002 [Plan:1.2] Add network roles, session resources, player slots, stable network IDs, and connecting/lobby states in `src/network/mod.rs`, `src/network/session.rs`, `src/main.rs`, and `src/states.rs`.

### Phase 2: User story 1 — Host and join a two-player session

#### Step 2.1: [Cross-cutting] Implement direct-IP session lifecycle
- **Requirements**: REQ-001, REQ-002, REQ-006
- **Description**: Let the host bind a configurable UDP port and let a guest connect to an entered IP:port. Validate protocol version and capacity, assign the two player slots, start the run from the host, and surface connection failures and timeouts.

#### Step 2.2: [Cross-cutting] Route independent player input
- **Requirements**: REQ-003
- **Description**: Associate each player entity with a stable owner/slot. The host reads its local controls; the guest sends sequenced movement, aim, sprint, fire, and reload input. Validate ownership, clamp input values, and reject messages for unknown players.

**Tasks**

- [ ] T003 [US1] [Plan:2.1] Implement UDP host binding, direct IP:port guest connection, version/capacity handshake, timeout, and session-start flow in `src/network/session.rs`.
- [ ] T004 [US1] [Plan:2.2] Assign host and guest player ownership; send sequenced input snapshots and validate/clamp remote inputs in `src/network/session.rs` and `src/network/protocol.rs`.

### Phase 3: User story 2 — Play a shared authoritative run

#### Step 3.1: [Cross-cutting] Make gameplay systems multi-player and host-authoritative
- **Requirements**: REQ-001, REQ-003, REQ-004
- **Description**: Replace one-player `Single` assumptions in player movement, aiming, weapons, camera, and related systems with per-player queries. Run gameplay simulation only on the host; clients must not independently simulate enemies, damage, or combat outcomes.

#### Step 3.2: [Cross-cutting] Target and spawn around both players
- **Requirements**: REQ-004, REQ-005
- **Description**: Update enemy AI to select a valid living player target, send damage to that player's entity, and avoid spawning enemies on either player. Preserve existing collision and damage rules under the host simulation.

#### Step 3.3: [Cross-cutting] Replicate the host's gameplay state
- **Requirements**: REQ-004, REQ-005
- **Description**: Replicate player and enemy transforms, health, weapon state, projectile state, run stats, and entity spawn/despawn events using stable IDs. Use reliable delivery for session/lifecycle events and sequenced or unreliable delivery for frequent state updates.

#### Step 3.4: [Cross-cutting] Smooth client-side presentation
- **Requirements**: REQ-004, REQ-005
- **Description**: Predict the guest's own movement from its input history, reconcile against host acknowledgements, and interpolate remote player/enemy states. Keep gameplay outcomes host-owned; do not add rollback or lag compensation in this release.

**Tasks**

- [ ] T005 [US2] [Plan:3.1] Refactor player movement, aim, reload, and firing to process multiple players and gate state-changing gameplay on the host in `src/player.rs`, `src/weapon.rs`, and `src/main.rs`.
- [ ] T006 [US2] [Plan:3.2] Update enemy targeting/spawn avoidance and damage recipients for both players in `src/enemy.rs` and `src/combat.rs`.
- [ ] T007 [US2] [Plan:3.3] Replicate stable-ID player/enemy/projectile state and spawn/despawn events in `src/network/replication.rs`.
- [ ] T008 [US2] [Plan:3.4] Add local input prediction, host acknowledgement/reconciliation, and remote interpolation in `src/network/replication.rs`.

### Phase 4: User story 3 — Connect, understand, and recover from session changes

#### Step 4.1: [Cross-cutting] Add host/join and per-player presentation
- **Requirements**: REQ-002, REQ-003, REQ-006
- **Description**: Add host and join-by-IP UI to the existing menu flow, show connection/lobby status, and provide player-specific HUD data. On each machine, follow the local player with the camera while rendering both players; ensure each player has independent aim/gun visuals.

#### Step 4.2: [Cross-cutting] Handle disconnects and invalid traffic
- **Requirements**: REQ-006, REQ-007
- **Description**: Bound packet sizes and input rates; reject invalid protocol/session/ownership data. On guest disconnect, notify the host and allow the host to continue solo; on host disconnect, return the guest to the menu with an explicit reason. Reconnection is out of scope.

**Tasks**

- [ ] T009 [US3] [Plan:4.1] Add host/join-by-IP controls, lobby status, player-specific HUD, and local-player camera/gun visuals in `src/ui.rs`, `src/camera.rs`, and `src/player.rs`.
- [ ] T010 [US3] [Plan:4.2] Implement bounded packet/input handling and the defined guest/host disconnect transitions in `src/network/session.rs`, `src/network/protocol.rs`, and `src/states.rs`.

### Phase 5: Cross-cutting validation

#### Step 5.1: [Cross-cutting] Add network and gameplay regression tests
- **Requirements**: REQ-007
- **Description**: Test protocol validation, slot assignment, input ownership, duplicate/stale input handling, host-only damage and enemy simulation, player-target selection, and disconnect transitions. Add loopback host-client tests for a complete two-player session.

#### Step 5.2: [Cross-cutting] Validate real direct-IP play
- **Requirements**: REQ-002, REQ-004, REQ-006, REQ-007
- **Description**: Run two game processes on one machine and two machines on a LAN, then test over the internet with the host UDP port forwarded. Exercise latency, packet loss, guest disconnect, and host shutdown; document the port and required router setup in the README.

**Tasks**

- [ ] T011 [Plan:5.1] Add protocol, ownership, host-authority, enemy-targeting, disconnect, and loopback integration tests under `src/network/` and the relevant module test suites.
- [ ] T012 [Plan:5.2] Validate loopback, LAN, and forwarded-port internet sessions, then document the direct-IP UDP port and setup requirements in `README.md`.

## Project Structure

```text
src/
  network/
    mod.rs              # network plugin and mode resources
    protocol.rs         # versioned messages and stable network IDs
    session.rs          # host/client lifecycle and input routing
    replication.rs      # host snapshots and client state application
  main.rs               # register networking plugin
  states.rs             # connecting/lobby/session transitions
  player.rs             # per-player input, movement, camera-facing visuals
  weapon.rs             # host-owned per-player firing and reload
  enemy.rs              # host target selection and spawning
  combat.rs              # authoritative collision/damage
  camera.rs              # local-player camera
  ui.rs                  # host/join flow and player-specific HUD
```

## Testing Strategy

- **appType**: Native desktop game with a host-client network session.
- **Critical user journeys**:
  1. Host starts a session and a guest joins via IP:port.
  2. Both players move, aim, fire, reload, take damage, and see the same enemy/run state.
  3. A guest disconnects and the host can continue; host shutdown returns the guest to the menu.
- **primaryValidationStack**: `cargo test`, protocol tests, and two-process loopback integration tests.
- **fallbackMatrix**:
  - `network-tier`: loopback host/client integration tests -> two-machine LAN -> internet direct-IP test with port forwarding.
- **Environment requirements**: Rust toolchain; two game processes for loopback; two machines and router access for internet validation.
- **knownGaps**: Internet testing without a reachable/forwarded UDP port cannot prove direct-IP connectivity. No relay, NAT traversal, or broad hostile-network security audit is in scope.
- **Test data strategy**: Use deterministic test inputs and fixed test session IDs; tear down host/client apps after each integration case.
- **Acceptance criteria**: A two-player run has one authoritative simulation; both clients observe the same host-confirmed combat and entity state; invalid/stale client input cannot directly author gameplay outcomes; connection failure and disconnect states are visible and recover to a defined state.
- **Validation review expectations**: Confirm both peers agree on player ownership and run state; verify host-only simulation; verify no system silently assumes exactly one `Player`.

## Requirement Mapping

| REQ ID | Description | Plan Items | Implementation Evidence |
|--------|-------------|-------------|--------------------------|
| REQ-001 | Host-authoritative sessions support two player slots. | 1.1, 1.2, 2.1, 3.1 | `src/network/`, `src/states.rs`, `src/player.rs`, `src/main.rs` |
| REQ-002 | A guest joins a host by direct IP and UDP port. | 1.1, 2.1, 4.1, 5.2 | `src/network/session.rs`, `src/network/protocol.rs`, `src/ui.rs`, `README.md` |
| REQ-003 | Each peer controls only its assigned player with independent input. | 1.2, 2.2, 3.1, 4.1 | `src/network/session.rs`, `src/player.rs`, `src/weapon.rs`, `src/ui.rs` |
| REQ-004 | The host owns gameplay simulation and clients render replicated state. | 1.2, 3.1, 3.3, 3.4 | `src/network/replication.rs`, `src/enemy.rs`, `src/combat.rs`, `src/player.rs` |
| REQ-005 | Both players participate in enemy targeting and host-confirmed combat. | 3.2, 3.3, 3.4 | `src/enemy.rs`, `src/combat.rs`, `src/network/replication.rs` |
| REQ-006 | Host/join status and disconnect outcomes are visible and defined. | 2.1, 4.1, 4.2, 5.2 | `src/network/session.rs`, `src/states.rs`, `src/ui.rs` |
| REQ-007 | Protocol, authority, disconnect, and end-to-end session behavior are tested. | 1.1, 3.3, 4.2, 5.1, 5.2 | `src/network/`, `src/*` test modules, loopback integration tests |
