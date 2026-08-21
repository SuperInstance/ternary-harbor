# Ternary Harbor — Agent Docking and Resource Management for Ternary Fleets

**Ternary Harbor** implements the harbor pattern for agent lifecycle management: agents arrive at rooms, get assigned berths (docks), receive guidance from pilots (local helpers), are rebalanced by tugs, and are protected by breakwaters (failure isolation). Each agent's priority is classified ternarily as `{-1 (reject/deprioritized), 0 (neutral), +1 (priority)}`, enabling three-tier scheduling.

## Why It Matters

Fleet resource management is fundamentally a scheduling problem: agents arrive, need compute resources, must be isolated from failures, and eventually depart. The harbor metaphor captures all these concerns naturally: berths have capacity limits, pilots guide agents into berths, tugs handle load balancing and berth-to-berth transfers, and breakwaters model which agents are sheltered versus evicted when a failure ("storm") arrives. The ternary priority system adds nuance missing from binary priority: positive-priority agents jump the queue, neutral agents queue normally, and negative agents queue normally as well — only `Positive` is fast-tracked.

## How It Works

### Berth Management

Each `Dock` (berth) is in one of four states: `Empty`, `Occupied(agent)`, `Reserved(agent)`, or `Maintenance`. Berths also track a `current_load` against a fixed `capacity`. Docking an agent:

1. Find an empty dock with sufficient remaining capacity.
2. Transition status: `Empty -> Occupied` (or `Empty -> Reserved -> Occupied` if pre-reserved for that agent).

Undocking transitions `Occupied -> Empty` and resets the berth's load to zero. (A berth only enters `Maintenance` via an explicit `start_maintenance()` call on an empty berth; it is **not** automatic on undock.)

### Pilot Service

A `Pilot` is a local helper that guides incoming agents into the first available berth (`guide_in`) and guides them back out of a specific berth (`guide_out`). A pilot is a single coordinator object, not a pool, so "pilot assignment" is a constant-time operation; the only linear cost is the berth scan.

### HarborMaster (Queued Scheduling)

The `HarborMaster` owns a queue of `DockingRequest`s. `request_docking` tries to dock immediately; if no berth with sufficient capacity is free, the request is enqueued. `Positive`-priority requests are pushed to the **front** of the queue; `Neutral` and `Negative` requests are pushed to the back. `process_queue` drains the queue, docking as many waiting agents as berths allow.

### Tug (Load Balancing & Transfers)

A `Tug` transfers load between two docks (`transfer_load`) and moves an agent from one berth to another (`assist_move`). Moves are atomic with respect to failure: if the destination cannot accept the agent, the source berth is left untouched, so an agent can never be stranded "homeless" by a failed transfer.

### Breakwater (Failure Isolation)

A `Breakwater` tracks the set of agents registered for protection, up to a fixed `capacity`. When `signal_storm(docked_agents)` is called, it partitions the docked agents: those **both registered and docked** are `sheltered` (up to capacity); all other docked agents are `evicted`. The returned `StormReport` enumerates both groups. (`register` already enforces the capacity bound, so every registered-and-docked agent is sheltered.) `calm()` clears the storm flag.

### Ternary Priority

Agent priority affects queue ordering in `HarborMaster`:

- **`Positive` (+1)**: pushed to the front of the queue — served before neutral/negative agents.
- **`Neutral` (0)**: pushed to the back — standard FIFO.
- **`Negative` (-1)**: pushed to the back, treated the same as neutral for ordering.

All three priorities attempt immediate docking first; only when queuing does priority affect order.

## Quick Start

```rust
use ternary_harbor::{Harbor, HarborMaster, AgentId, BerthId, DockingRequest, PilotResult, Ternary};

let mut harbor = Harbor::new("main", 4, 100);
let mut hm = HarborMaster::new("main");

// Submit a high-priority docking request; it docks immediately.
let result = hm.request_docking(
    &mut harbor,
    DockingRequest::new(AgentId(1), Ternary::Positive, 20),
);
assert_eq!(result, PilotResult::Docked(BerthId(0)));
assert_eq!(hm.queue_length(), 0);
```

```bash
cargo add ternary-harbor
```

## API

| Type / Function | Description |
|---|---|
| `Dock` | Single berth: `new(id, capacity)`, `status()`, `current_load()`, `available_capacity()`, `dock()`, `reserve()`, `undock()`, `start_maintenance()`, `add_load()` |
| `BerthStatus` | `Empty`, `Occupied(AgentId)`, `Reserved(AgentId)`, `Maintenance` |
| `BerthId`, `AgentId` | Newtype identifiers |
| `Harbor` | Collection of berths: `new(name, count, capacity)`, `find_empty()`, `available_berths()`, `total_load()` |
| `Pilot` | `guide_in()` docks into the first empty berth; `guide_out()` undocks a specific berth |
| `HarborMaster` | `request_docking()`, `process_queue()`, `queue_length()` |
| `DockingRequest` | `new(agent, priority: Ternary, requested_load)` |
| `Ternary` | Priority: `Negative`, `Neutral`, `Positive` |
| `Tug` | `transfer_load()` between docks; `assist_move()` between berths |
| `Breakwater` | `register()`, `unregister()`, `signal_storm()`, `calm()` |
| `StormReport` | `{ sheltered: Vec<AgentId>, evicted: Vec<AgentId> }` |

## Architecture Notes

Harbors manage agent lifecycle at **SuperInstance** room boundaries. Each room has a harbor that processes incoming agents, assigns them to compute resources, and protects against cascading failures. See [Architecture](https://github.com/SuperInstance/SuperInstance/blob/main/ARCHITECTURE.md).

## References

- Tanenbaum, Andrew. *Distributed Systems*, 4th ed., 2023 — resource management.
- Brewer, Eric. "CAP Twelve Years Later," *IEEE Computer*, 45(2), 2012 — fault tolerance.
- Kleinberg, Jon & Tardos, Éva. *Algorithm Design*, Pearson, 2006 — scheduling algorithms.

## Complexity Summary

| Operation | Time | Notes |
|---|---|---|
| Dock assignment (`guide_in` / `request_docking`) | O(d) for d berths | Linear scan for an empty berth with capacity |
| Enqueue | O(1) | `VecDeque` `push_front` (positive) / `push_back` (others) |
| `process_queue` | O(q · d) for q queued requests | Each request rescans berths |
| `signal_storm` | O(n · p) | n docked agents × p protected (linear membership check) |
| Load transfer (`Tug::transfer_load`) | O(1) | Direct dock mutation after pre-checks |

The harbor scales linearly with berth count; there is no heap or balanced tree involved in scheduling — priority ordering is achieved with a double-ended queue.

## License

MIT
