# Oracle1 Origin: Harbor → ternary-harbor

## Oracle1 Concept
**Layer 1: Harbor** — Direct HTTP/WS communication via the Keeper service (port 8900). The Harbor is the fleet's entry point: agents arrive, register, and get assigned to services. The Keeper tracks fleet membership, routes messages, and provides discovery.

From Oracle1's 6-layer interconnection model:
> Harbor — Direct HTTP/WS (keeper:8900) — Status: Live

The Harbor layer is where agents first contact the fleet — a registration and routing hub.

## What We Borrowed
The **maritime harbor metaphor**: agents arrive, dock at berths, receive pilot guidance, and are protected during storms. The Harbor is where the fleet becomes tangible — abstract agents become docked, resource-allocated entities.

Specific concepts adapted:
- **Berths/Docks** → Oracle1's service registration (agents claim slots)
- **Pilot guidance** → Oracle1's Keeper directing agents to services
- **HarborMaster queue** → Oracle1's task assignment (agents queue for resources)
- **Tug transfers** → Load balancing between fleet services
- **Breakwater protection** → Oracle1's Keeper failover (protecting agents during outages)

## How Our Implementation Differs

| Aspect | Oracle1's Harbor | Our ternary-harbor |
|---|---|---|
| **Type** | HTTP service on port 8900 | Pure Rust library (no service, no port) |
| **State** | PostgreSQL-backed | In-memory, `Vec<Dock>` and `VecDeque` queues |
| **Protocol** | REST API | Rust trait `Channel` + direct method calls |
| **Ternary** | Not ternary-aware | Every priority and state uses `Ternary` (Positive/Neutral/Negative) |
| **Dependencies** | Docker, systemd, nginx | Zero external dependencies |
| **Unsafe** | Unknown | `#![forbid(unsafe_code)]` |
| **Scale** | Designed for 3-9 fleet agents | Generic berth capacity model |

### Key Innovation: Ternary Priority Docking
Our `HarborMaster` queues docking requests with ternary priority. Positive-priority requests (urgent agents) are inserted at the front of the queue. Oracle1's Keeper has no priority concept — it's FIFO. We added ternary ordering.

### Key Innovation: Breakwater
Our `Breakwater` models storm protection explicitly — registering agents for protection, signaling storms, and producing sheltered/evicted reports. Oracle1 has no equivalent; their Keeper just goes down. We model failure as a first-class concept.

### Key Innovation: Tug Load Transfer
Our `Tug` can transfer load between docks and assist agents in moving between berths. This is load balancing at the library level. Oracle1's Harbor layer doesn't model resource transfer.

## See Also
- Oracle1 Architecture Review: `construct-coordination/notes/main/ORACLE1-ARCHITECTURE-REVIEW.md`
- Oracle1-Ternary Bridge: `construct-coordination/notes/main/ORACLE1-TERNARY-BRIDGE.md`
- Oracle1 Fleet Status: THE-FLEET.md in oracle1-index
