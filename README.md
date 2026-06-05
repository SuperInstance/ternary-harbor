# ternary-harbor

**Where agents dock. Berths, pilots, and the logistics of arrival.**

A harbor is the space between the open ocean and the safety of port. Ships arrive from voyages, wait for a berth, get guided in by a pilot, dock, unload cargo, refit, and eventually depart again. The harbor manages the entire lifecycle of arrival and departure — it's the coordination layer between the outside world and the protected interior.

In the SuperInstance fleet, a harbor is where agent processes connect to rooms. An agent arriving at a harbor requests a berth (an allocation of resources). If space is available, a pilot (a guided setup process) helps the agent dock. Once docked, the agent can exchange cargo (data) with the room. When done, the agent undocks and departs.

## What's Inside

- **`Harbor`** — manages berths, pilots, and docking schedule
- **`Berth`** — a docking slot with capacity, assigned agent, and status
- **`Pilot`** — a guided setup process that helps agents dock safely
- **`dock(harbor, agent)`** — request a berth and begin docking
- **`undock(harbor, agent_id)`** — release the berth and depart
- **`berth_status(harbor)`** — which berths are occupied, available, or reserved
- **`pilot_guide(harbor, agent_id)`** — get the pilot's setup instructions

## Quick Example

```rust
use ternary_harbor::*;

let mut harbor = Harbor::new(5); // 5 berths available

// Agent 42 arrives and requests docking
let result = dock(&mut harbor, 42);
assert!(result.is_ok());

// Check berth status
let status = berth_status(&harbor);
println!("Available: {}/5", status.available);

// Agent 42 departs
undock(&mut harbor, 42);
// Berth released, available for next arrival
```

## The Deeper Truth

**Harbors are the coordination bottleneck.** Every agent that wants to join a room must pass through the harbor. This means the harbor controls the admission rate — it can accept agents faster than the room can handle, creating a queue; or it can reject agents when full, creating a barrier. The harbor is the *shaper* of the room's population dynamics.

The pilot pattern is borrowed from maritime tradition: real harbor pilots board ships at the harbor entrance and guide them through the narrow channel to their berth. The pilot knows the local conditions (current, depth, traffic) in a way the ship's captain doesn't. In the fleet, the pilot guides the agent through setup — configuring its state, connecting it to the room's communication channels, and verifying it meets the room's requirements.

**Use cases:**
- **Agent lifecycle management** — control how agents join and leave rooms
- **Resource allocation** — manage limited berths (memory, compute, connections)
- **Service discovery** — agents dock at the harbor to discover room capabilities
- **Load balancing** — distribute agents across multiple harbors
- **Infrastructure coordination** — the harbor pattern for microservice mesh

## See Also

- **ternary-room** — rooms are what harbors connect to
- **ternary-anchor** — anchors hold position once docked
- **ternary-cargo** — cargo is what agents exchange while docked
- **ternary-dockyard** — dockyards repair agents between voyages
- **ternary-shipyard** — shipyards build the agents that arrive at harbors
- **ternary-beacon** — beacons announce harbor locations
- **ternary-navigator** — navigation guides agents to harbors

## Install

```bash
cargo add ternary-harbor
```

## License

MIT
