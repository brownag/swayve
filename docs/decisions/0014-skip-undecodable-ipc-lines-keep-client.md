---
id: "0014"
title: "Skip Non-UTF-8 IPC Lines and Keep the Client Connected"
status: "Accepted" # Proposed | Accepted | Deprecated | Superseded
date: 2026-11-08
authors:
  - "Andrew G. Brown"
supersedes: null
superseded_by: null
source_context: "Orchestration 21 (EXECUTION_PLAN_21.md D1, G35; commit 235adf3); src/daemon.rs handle_client; tests test_non_utf8_ipc_line_is_skipped, test_malformed_ipc_line_keeps_client"
---

# 14. Skip Non-UTF-8 IPC Lines and Keep the Client Connected

## Context and Problem Statement
The daemon reads newline-delimited JSON from each client ([0004](0004-newline-delimited-json-ipc.md)). Before this change, `handle_client` read lines with `lines().next_line()`. A line that was not valid UTF-8 made that call return an error, which closed the whole connection. One bad byte in one message therefore dropped every later message from that client, including valid ones. Malformed JSON was already handled differently: the line is skipped and the client stays connected (G26, Orchestration 19). The two cases should follow the same rule.

## Decision Drivers
* One bad line must not cost a client its connection or the messages after it.
* Input handling should be the same for every kind of bad line.
* Clients are fire-and-forget (see [0009](0009-thin-socket-integrations.md)). The daemon has no response channel that could report the error.

## Considered Options
1. Close the client on a non-UTF-8 line (previous behavior).
2. Log the line, skip it, and keep the client connected.
3. Decode lossily (`String::from_utf8_lossy`) and pass the result to the JSON parser, which will reject it.

## Decision Outcome
Chosen option: option 2. `handle_client` reads raw bytes with `read_until(b'\n')` and converts each line with `std::str::from_utf8`. On failure it logs `[daemon] Skipping non-UTF-8 IPC line` and continues. The client stays connected. Malformed JSON keeps its existing handling. Option 3 was rejected because a lossy conversion can turn a corrupt line into text that parses as a valid message, which would apply an unintended state change.

### Consequences
* **Positive**: one bad line no longer ends a client session. Later valid messages still reach the daemon.
* **Positive**: non-UTF-8 input and malformed JSON now follow one rule, so clients have one thing to learn.
* **Negative**: a skipped line is lost. The only record is the daemon's log line. A client whose encoder produces invalid UTF-8 gets no error back.
* **Negative**: a flood of bad bytes from one client keeps that connection open and costs the daemon a log line per line. There is no rate limit.

## Validation & Compliance
* `src/daemon.rs`: `test_non_utf8_ipc_line_is_skipped` sends `b"\xff\xfe\n"` followed by a valid `daemon.focus` line and expects the `Focus` event within 2 s.
* `src/daemon.rs`: `test_malformed_ipc_line_keeps_client` covers the same rule for malformed JSON.
* This ADR supersedes the "non-UTF-8 lines still close the client" limit recorded for G26 in `planning/GOALS.md`.
