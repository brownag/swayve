---
id: "0015"
title: "Per-Client IPC Rate Limit: Drop and Log"
status: "Accepted" # Proposed | Accepted | Deprecated | Superseded
date: 2026-11-08
authors:
  - "Andrew G. Brown"
supersedes: null
superseded_by: null
source_context: "Orchestration 22 (EXECUTION_PLAN_22.md D1, G43; commit 5d7ef87); src/ratelimit.rs; src/daemon.rs handle_client; tests/slow_consumer.rs"
---

# 15. Per-Client IPC Rate Limit: Drop and Log

## Context and Problem Statement
The daemon reads newline-delimited JSON from each client ([0004](0004-newline-delimited-json-ipc.md)). Before this change, nothing limited how fast one client could send lines. A client that floods the socket takes daemon time and produces broadcasts for every subscriber. [0014](0014-skip-undecodable-ipc-lines-keep-client.md) noted this as a negative consequence: a flood of bad bytes costs one log line per line, and there was no rate limit.

## Decision Drivers
* One client must not be able to starve the daemon or the other clients by sending faster than any real agent does.
* Clients are fire-and-forget (see [0009](0009-thin-socket-integrations.md)). The daemon has no response channel that could report the limit.
* The limit should be a safety net for bursts, not a tuned setting that users must configure.

## Considered Options
1. No limit (previous behavior).
2. A token bucket per connection; over-limit lines dropped and logged; the client stays connected.
3. A token bucket per connection; over-limit clients disconnected.
4. A per-client limit read from the config file.

## Decision Outcome
Chosen option: option 2. Each client connection gets a token bucket (`src/ratelimit.rs`): a sustained rate of 100 lines per second and a burst of 200. `handle_client` checks the bucket after each line is read. An over-limit line is dropped, the daemon logs `[daemon] Rate limit exceeded; dropping IPC line`, and the client stays connected. The limits are constants, not config. Option 3 was rejected for the same reason as the skip policy in [0014](0014-skip-undecodable-ipc-lines-keep-client.md): a client that loses its connection also loses later valid messages. Option 4 was rejected because no user has asked to tune it, and a config knob adds surface before there is a need.

### Consequences
* **Positive**: one client cannot flood the daemon. The daemon's work per client is bounded by the bucket.
* **Positive**: a dropped line does not end the session, so later valid messages still arrive.
* **Negative**: a legitimate burst of more than 200 lines from one client in a short time loses the excess lines. No real integration in this repo sends that many at once.
* **Negative**: a dropped line is lost. The only record is the daemon's log line. A client that exceeds the limit gets no error reply.
* **Negative**: the limit is a constant. Changing it needs a rebuild, not a config edit.

## Validation & Compliance
* `src/ratelimit.rs`: `test_rate_limiter_allows_burst_then_denies`, `test_rate_limiter_refills_over_time` and `test_rate_limiter_caps_at_capacity` use injected `Instant` values and sleep for no time.
* `tests/slow_consumer.rs` sends its 300 status updates across three connections, no connection sends more than 150 lines, so the limit does not drop them. The assertions are unchanged.
* `tests/integration_tests.rs` and `tests/subscribe_golden.rs` send far fewer lines per connection than the burst, so they are unaffected.
