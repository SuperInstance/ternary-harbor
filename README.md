# Ternary Harbor — Agent Docking and Resource Management for Ternary Fleets

**Ternary Harbor** implements the harbor pattern for agent lifecycle management: agents arrive at rooms, get assigned berths (docks), receive guidance from pilots (local helpers), and are protected by breakwaters (failure isolation). Each agent's priority is classified ternarily as {-1 (reject), 0 (neutral), +1 (priority)}, enabling three-tier scheduling.

## Why It Matters

Fleet resource management is fundamentally a scheduling problem: agents arrive, need compute resources, must be isolated from failures, and eventually depart. The harbor metaphor captures all these concerns naturally: berths have capacity limits, pilots provide initialization assistance, tugs handle resource-constrained agents, and breakwaters prevent cascading failures from reaching other docked agents. The ternary priority system adds nuance missing from binary priority: agents can be deprioritized (-1), treated normally (0), or fast-tracked (+1) without the complexity of a continuous priority score.

## How It Works

### Berth Management

Each `Dock` (berth) has a status: `Empty`, `Occupied(agent)`, `Reserved(agent)`, or `Maintenance`. Capacity limits ensure a dock isn't overloaded. Docking an agent:
1. Find an empty dock with sufficient capacity
2. Transition status: Empty → Reserved → Occupied
3. Assign pilot if agent needs initialization assistance

Undocking transitions: Occupied → Empty (or Maintenance if cleanup needed).

### Pilot Service

Pilots are local helper agents that guide incoming agents through initialization:
- Context injection (loading room state, constellation configuration)
- Dependency resolution (ensuring required skills are available)
- Health check (verifying the agent can execute its task)

Pilot assignment is O(p) for p available pilots.

### Breakwater (Failure Isolation)

Breakwaters monitor docked agents for failure signals. When an agent enters a failure state, the breakwater:
1. Quarantines the agent (prevents it from sending messages)
2. Logs diagnostic data
3. Notifies the steward for resource cleanup

This prevents cascading failures — the "tsunami" effect — where one agent's failure destabilizes its neighbors.

### Ternary Priority

Agent priority determines scheduling order:
- **+1 (Positive)**: High-priority agents dock first, get largest berths
- **0 (Neutral)**: Standard scheduling
- **-1 (Negative)**: Deprioritized; docked only when capacity is available

## Quick Start

```rust
use ternary_harbor::{Dock, BerthId, AgentId, BerthStatus};

let mut dock = Dock::new(BerthId(1), 100);
assert_eq!(dock.capacity(), 100);

// Dock an agent
let agent = AgentId(42);
// dock.reserve(agent);
// dock.arrive(agent);
```

```bash
cargo add ternary-harbor
```

## API

| Type / Function | Description |
|---|---|
| `Dock` | Single berth: `new(id, capacity)`, `status()`, `current_load()` |
| `BerthStatus` | `Empty`, `Occupied(AgentId)`, `Reserved(AgentId)`, `Maintenance` |
| `BerthId`, `AgentId` | Newtype identifiers |
| `Ternary` | Priority: `Negative`, `Neutral`, `Positive` |

## Architecture Notes

Harbors manage agent lifecycle at **SuperInstance** room boundaries. Each room has a harbor that processes incoming agents, assigns them to compute resources, and protects against cascading failures. The γ + η = C conservation manifests in berth utilization: active agents contribute γ (growth), idle agents contribute η (entropy overhead), and the harbor balances the total load against room capacity C. See [Architecture](https://github.com/SuperInstance/SuperInstance/blob/main/ARCHITECTURE.md).

## References

- Tanenbaum, Andrew. *Distributed Systems*, 4th ed., 2023 — resource management.
- Brewer, Eric. "CAP Twelve Years Later," *IEEE Computer*, 45(2), 2012 — fault tolerance.
- Kleinberg, Jon & Tardos, Éva. *Algorithm Design*, Pearson, 2006 — scheduling algorithms.

## License

MIT
