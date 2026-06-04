# ternary-harbor — Harbor pattern for agent docking and resource management

Harbor models how agents arrive at rooms, get assigned berths, receive pilot assistance, and are protected during failure storms. Every agent in a fleet needs a place to dock; this crate makes that explicit.

## Why This Exists

In a multi-room fleet, agents don't just teleport into place. They arrive, request resources, wait in queues if space is full, need guidance (pilots) to settle in, and occasionally need to be moved (tugs) or protected (breakwaters) when things go wrong. The Harbor pattern makes this lifecycle visible and testable, inspired by Oracle1's Harbor interconnection layer.

## Core Concepts

- **Harbor** — A collection of docking berths (rooms). Think of it as a port where agents dock.
- **Dock** — A single berth that holds one agent at a time. Has a resource capacity (load).
- **HarborMaster** — Manages a priority queue of docking requests and assigns berths. Positive-priority agents get to the front of the queue.
- **Pilot** — Guides agents into and out of berths. The actual docking/undocking operation.
- **Tug** — Assists with load transfers between docks and agent relocations between berths.
- **Breakwater** — Protects registered agents during storms (failure events). Agents not registered or beyond capacity get evicted.
- **Ternary** — Three-valued priority: Positive (high), Neutral (normal), Negative (low).

## Quick Start

```toml
[dependencies]
ternary-harbor = "0.1"
```

```rust
use ternary_harbor::*;

let mut harbor = Harbor::new("main-room", 4, 100);
let mut master = HarborMaster::new("main-room");

// Submit docking requests with ternary priority
let req = DockingRequest::new(AgentId(1), Ternary::Positive, 30);
let result = master.request_docking(&mut harbor, req);

// Use a tug to transfer resources between docks
let tug = Tug::new(1);
tug.transfer_load(harbor.dock_mut(BerthId(0)).unwrap(),
                   harbor.dock_mut(BerthId(1)).unwrap(), 15).unwrap();

// Protect agents during a storm with a breakwater
let mut bw = Breakwater::new(10);
bw.register(AgentId(1)).unwrap();
let report = bw.signal_storm(&[AgentId(1), AgentId(2)]);
```

## API Overview

| Type | Description |
|------|-------------|
| `Harbor` | Collection of docks with name and capacity tracking |
| `Dock` | Single berth for one agent, with load management |
| `BerthStatus` | Empty, Occupied, Reserved, or Maintenance |
| `HarborMaster` | Priority queue manager for docking requests |
| `Pilot` | Guides agents into/out of berths |
| `Tug` | Transfers load between docks, assists agent moves |
| `Breakwater` | Protects registered agents during storm events |
| `DockingRequest` | Queued request with agent ID, priority, and load |

## How It Works

Harbor uses a straightforward queue-based assignment system. The `HarborMaster` maintains a `VecDeque` of docking requests. Positive-priority requests are pushed to the front of the queue; neutral and negative go to the back. When `process_queue` is called, it tries to assign berths in queue order.

Each `Dock` tracks a resource load (u32) against a capacity. Agents consume resources when docked; the `Tug` can transfer load between docks without undocking. Docks have a maintenance mode that blocks all operations.

`Breakwater` is a separate concern: it maintains a list of protected agent IDs. When a storm is signaled, agents on the protected list (up to capacity) are sheltered; others are reported as evicted. This models circuit-breaker-style failure isolation.

## Known Limitations

- Harbor assignment is first-fit, not optimal packing. An agent requesting 80 units might fail on a dock with 50 available even if another dock has 90 free.
- The breakwater's protection list is static — agents must be registered before the storm hits. There's no dynamic escalation.
- No timeout on reservations. A reserved berth stays reserved until the expected agent docks or is manually cleared.
- Queue priority is binary (positive = front, else = back). No gradations within neutral/negative.

## Use Cases

- **Room assignment** — Agents arriving at a fleet room request a berth with resource requirements. HarborMaster queues them if the room is full.
- **Resource rebalancing** — A Tug transfers load from overloaded docks to underloaded ones, keeping resource usage even.
- **Failure isolation** — Breakwater protects critical agents during cascading failures while evicting non-essential ones.
- **Maintenance scheduling** — Docks can be put into maintenance mode, preventing new docking while existing agents are moved out.

## Ecosystem Context

Part of the SuperInstance ternary fleet ecosystem. Relates to `ternary-beacon` (agents discover harbors via beacons), `ternary-channel` (agents communicate about docking status), and `ternary-room` (the room abstraction that contains harbors).

## License

MIT
