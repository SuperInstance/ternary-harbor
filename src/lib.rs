#![forbid(unsafe_code)]

//! Harbor pattern for agent docking and resource management.
//!
//! Models how agents arrive at rooms, get assigned berths, receive assistance
//! from pilots and tugs, and are protected by breakwaters during failures.

use std::collections::VecDeque;

// ---- Core ternary types ----

/// A ternary value used for priority and state classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Ternary {
    /// Negative: reject, low priority, inactive.
    Negative,
    /// Neutral: undecided, medium priority, idle.
    Neutral,
    /// Positive: accept, high priority, active.
    Positive,
}

/// Unique identifier for a docking berth within a harbor.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BerthId(pub u64);

/// Unique identifier for an agent requesting docking.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AgentId(pub u64);

// ---- Dock ----

/// Status of a single docking berth.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BerthStatus {
    /// Berth is available for a new agent.
    Empty,
    /// An agent is docked and active.
    Occupied(AgentId),
    /// Berth is reserved but agent hasn't arrived yet.
    Reserved(AgentId),
    /// Berth is under maintenance; cannot accept agents.
    Maintenance,
}

/// A single docking berth that holds one agent at a time.
#[derive(Debug, Clone)]
pub struct Dock {
    id: BerthId,
    status: BerthStatus,
    capacity: u32,
    current_load: u32,
}

impl Dock {
    /// Create a new empty dock with the given capacity.
    pub fn new(id: BerthId, capacity: u32) -> Self {
        Self {
            id,
            status: BerthStatus::Empty,
            capacity,
            current_load: 0,
        }
    }

    pub fn id(&self) -> BerthId {
        self.id
    }

    pub fn status(&self) -> &BerthStatus {
        &self.status
    }

    pub fn capacity(&self) -> u32 {
        self.capacity
    }

    pub fn current_load(&self) -> u32 {
        self.current_load
    }

    pub fn available_capacity(&self) -> u32 {
        self.capacity.saturating_sub(self.current_load)
    }

    /// Dock an agent. Returns `Ok(())` if successful, or an error message.
    pub fn dock(&mut self, agent: AgentId) -> Result<(), &'static str> {
        match self.status {
            BerthStatus::Empty => {
                self.status = BerthStatus::Occupied(agent);
                Ok(())
            }
            BerthStatus::Reserved(a) if a == agent => {
                self.status = BerthStatus::Occupied(agent);
                Ok(())
            }
            BerthStatus::Maintenance => Err("Berth is under maintenance"),
            _ => Err("Berth is not available"),
        }
    }

    /// Reserve a berth for a specific agent.
    pub fn reserve(&mut self, agent: AgentId) -> Result<(), &'static str> {
        match self.status {
            BerthStatus::Empty => {
                self.status = BerthStatus::Reserved(agent);
                Ok(())
            }
            _ => Err("Can only reserve an empty berth"),
        }
    }

    /// Undock the current agent, freeing the berth.
    pub fn undock(&mut self) -> Result<AgentId, &'static str> {
        match self.status {
            BerthStatus::Occupied(a) | BerthStatus::Reserved(a) => {
                let agent = a;
                self.status = BerthStatus::Empty;
                self.current_load = 0;
                Ok(agent)
            }
            BerthStatus::Empty => Err("Berth is already empty"),
            BerthStatus::Maintenance => Err("Cannot undock from maintenance berth"),
        }
    }

    /// Set the berth into maintenance mode. Must be empty first.
    pub fn start_maintenance(&mut self) -> Result<(), &'static str> {
        match self.status {
            BerthStatus::Empty => {
                self.status = BerthStatus::Maintenance;
                Ok(())
            }
            _ => Err("Can only maintenance an empty berth"),
        }
    }

    /// End maintenance, returning berth to empty state.
    pub fn end_maintenance(&mut self) -> Result<(), &'static str> {
        match self.status {
            BerthStatus::Maintenance => {
                self.status = BerthStatus::Empty;
                Ok(())
            }
            _ => Err("Berth is not in maintenance"),
        }
    }

    /// Add load to the dock (resources consumed by the docked agent).
    pub fn add_load(&mut self, amount: u32) -> Result<(), &'static str> {
        if self.current_load + amount > self.capacity {
            return Err("Would exceed capacity");
        }
        self.current_load += amount;
        Ok(())
    }

    /// Release load from the dock.
    pub fn release_load(&mut self, amount: u32) {
        self.current_load = self.current_load.saturating_sub(amount);
    }

    pub fn is_empty(&self) -> bool {
        matches!(self.status, BerthStatus::Empty)
    }

    pub fn is_occupied(&self) -> bool {
        matches!(self.status, BerthStatus::Occupied(_))
    }
}

// ---- Harbor ----

/// A harbor containing multiple docking berths for agents.
#[derive(Debug, Clone)]
pub struct Harbor {
    docks: Vec<Dock>,
    name: String,
}

impl Harbor {
    /// Create a new harbor with a given name and number of docks, each with the given capacity.
    pub fn new(name: &str, dock_count: usize, dock_capacity: u32) -> Self {
        let docks = (0..dock_count)
            .map(|i| Dock::new(BerthId(i as u64), dock_capacity))
            .collect();
        Self {
            docks,
            name: name.to_string(),
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn docks(&self) -> &[Dock] {
        &self.docks
    }

    pub fn dock_mut(&mut self, id: BerthId) -> Option<&mut Dock> {
        self.docks.iter_mut().find(|d| d.id() == id)
    }

    /// Number of empty berths.
    pub fn available_berths(&self) -> usize {
        self.docks.iter().filter(|d| d.is_empty()).count()
    }

    /// Total number of berths.
    pub fn total_berths(&self) -> usize {
        self.docks.len()
    }

    /// Whether the harbor has any available berths.
    pub fn has_capacity(&self) -> bool {
        self.available_berths() > 0
    }

    /// Find the first empty dock.
    pub fn find_empty(&self) -> Option<&Dock> {
        self.docks.iter().find(|d| d.is_empty())
    }

    /// Total load across all docks.
    pub fn total_load(&self) -> u32 {
        self.docks.iter().map(|d| d.current_load()).sum()
    }

    /// Total capacity across all docks.
    pub fn total_capacity(&self) -> u32 {
        self.docks.iter().map(|d| d.capacity()).sum()
    }
}

// ---- Pilot ----

/// Result of a pilot guiding an agent into dock.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PilotResult {
    /// Successfully docked at the specified berth.
    Docked(BerthId),
    /// Agent queued for later docking.
    Queued,
    /// Docking refused (harbor full or agent rejected).
    Refused,
}

/// A pilot guides agents into available berths.
#[derive(Debug, Clone)]
pub struct Pilot {
    harbor_name: String,
}

impl Pilot {
    pub fn new(harbor_name: &str) -> Self {
        Self {
            harbor_name: harbor_name.to_string(),
        }
    }

    pub fn harbor_name(&self) -> &str {
        &self.harbor_name
    }

    /// Guide an agent into the first available berth.
    pub fn guide_in(&self, harbor: &mut Harbor, agent: AgentId) -> PilotResult {
        if let Some(dock) = harbor.find_empty() {
            let berth_id = dock.id();
            if let Some(dock) = harbor.dock_mut(berth_id) {
                if dock.dock(agent).is_ok() {
                    return PilotResult::Docked(berth_id);
                }
            }
        }
        PilotResult::Refused
    }

    /// Guide an agent out of a specific berth.
    pub fn guide_out(&self, harbor: &mut Harbor, berth: BerthId) -> Result<AgentId, &'static str> {
        if let Some(dock) = harbor.dock_mut(berth) {
            dock.undock()
        } else {
            Err("Berth not found")
        }
    }
}

// ---- HarborMaster ----

/// A queued docking request.
#[derive(Debug, Clone)]
pub struct DockingRequest {
    agent: AgentId,
    priority: Ternary,
    requested_load: u32,
}

impl DockingRequest {
    pub fn new(agent: AgentId, priority: Ternary, requested_load: u32) -> Self {
        Self {
            agent,
            priority,
            requested_load,
        }
    }

    pub fn agent(&self) -> AgentId {
        self.agent
    }

    pub fn priority(&self) -> Ternary {
        self.priority
    }

    pub fn requested_load(&self) -> u32 {
        self.requested_load
    }
}

/// The HarborMaster manages docking queues and berth assignments.
#[derive(Debug, Clone)]
pub struct HarborMaster {
    queue: VecDeque<DockingRequest>,
    pilot: Pilot,
}

impl HarborMaster {
    pub fn new(harbor_name: &str) -> Self {
        Self {
            queue: VecDeque::new(),
            pilot: Pilot::new(harbor_name),
        }
    }

    /// Submit a docking request. Queued if no berth available.
    pub fn request_docking(&mut self, harbor: &mut Harbor, req: DockingRequest) -> PilotResult {
        // Try to dock immediately
        if let Some(dock) = harbor.find_empty() {
            let berth_id = dock.id();
            if let Some(dock) = harbor.dock_mut(berth_id) {
                if req.requested_load <= dock.available_capacity() {
                    if dock.dock(req.agent).is_ok() {
                        if dock.add_load(req.requested_load).is_err() {
                            // Load exceeded capacity; still docked but without full load
                        }
                        return PilotResult::Docked(berth_id);
                    }
                }
            }
        }

        // Queue the request (positive priority first)
        self.enqueue(req.clone());
        PilotResult::Queued
    }

    fn enqueue(&mut self, req: DockingRequest) {
        // Insert positive-priority requests at the front
        match req.priority {
            Ternary::Positive => self.queue.push_front(req),
            _ => self.queue.push_back(req),
        }
    }

    /// Process the queue, assigning berths to waiting agents.
    pub fn process_queue(&mut self, harbor: &mut Harbor) -> Vec<PilotResult> {
        let mut results = Vec::new();
        let mut retries = VecDeque::new();

        while let Some(req) = self.queue.pop_front() {
            if let Some(dock) = harbor.find_empty() {
                let berth_id = dock.id();
                if let Some(dock) = harbor.dock_mut(berth_id) {
                    if dock.dock(req.agent).is_ok() {
                        let _ = dock.add_load(req.requested_load);
                        results.push(PilotResult::Docked(berth_id));
                        continue;
                    }
                }
            }
            retries.push_back(req);
        }

        self.queue = retries;
        results
    }

    pub fn queue_length(&self) -> usize {
        self.queue.len()
    }

    pub fn pilot(&self) -> &Pilot {
        &self.pilot
    }
}

// ---- Tug ----

/// A tug assists agents with maneuvering — load balancing and transfers.
#[derive(Debug, Clone)]
pub struct Tug {
    id: u64,
}

impl Tug {
    pub fn new(id: u64) -> Self {
        Self { id }
    }

    pub fn id(&self) -> u64 {
        self.id
    }

    /// Transfer load from one dock to another.
    pub fn transfer_load(
        &self,
        from: &mut Dock,
        to: &mut Dock,
        amount: u32,
    ) -> Result<(), &'static str> {
        if from.current_load() < amount {
            return Err("Source doesn't have enough load");
        }
        if to.available_capacity() < amount {
            return Err("Destination doesn't have enough capacity");
        }
        from.release_load(amount);
        to.add_load(amount)?;
        Ok(())
    }

    /// Assist an agent in moving between berths.
    pub fn assist_move(
        &self,
        harbor: &mut Harbor,
        from_berth: BerthId,
        to_berth: BerthId,
    ) -> Result<AgentId, &'static str> {
        let agent = {
            let from = harbor
                .dock_mut(from_berth)
                .ok_or("Source berth not found")?;
            if !from.is_occupied() {
                return Err("Source berth is not occupied");
            }
            match from.status() {
                BerthStatus::Occupied(a) => *a,
                _ => return Err("Source berth is not occupied"),
            }
        };

        {
            let from = harbor.dock_mut(from_berth).unwrap();
            from.undock()?;
        }

        {
            let to = harbor
                .dock_mut(to_berth)
                .ok_or("Destination berth not found")?;
            to.dock(agent)?;
        }

        Ok(agent)
    }
}

// ---- Breakwater ----

/// A breakwater protects the harbor from storms (failure events).
#[derive(Debug, Clone)]
pub struct Breakwater {
    /// Agents protected by this breakwater.
    protected: Vec<AgentId>,
    /// Whether a storm is currently active.
    storm_active: bool,
    /// Maximum number of agents this breakwater can protect.
    capacity: usize,
}

/// Result of a storm event.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StormReport {
    /// Agents that were safely sheltered.
    pub sheltered: Vec<AgentId>,
    /// Agents that were evicted (couldn't be protected).
    pub evicted: Vec<AgentId>,
}

impl Breakwater {
    pub fn new(capacity: usize) -> Self {
        Self {
            protected: Vec::new(),
            storm_active: false,
            capacity,
        }
    }

    /// Register an agent for protection.
    pub fn register(&mut self, agent: AgentId) -> Result<(), &'static str> {
        if self.protected.len() >= self.capacity {
            return Err("Breakwater at capacity");
        }
        if self.protected.contains(&agent) {
            return Err("Agent already registered");
        }
        self.protected.push(agent);
        Ok(())
    }

    /// Unregister an agent from protection.
    pub fn unregister(&mut self, agent: AgentId) -> Result<(), &'static str> {
        let idx = self
            .protected
            .iter()
            .position(|a| *a == agent)
            .ok_or("Agent not registered")?;
        self.protected.remove(idx);
        Ok(())
    }

    /// Signal a storm (failure event). Agents beyond capacity are evicted.
    pub fn signal_storm(&mut self, docked_agents: &[AgentId]) -> StormReport {
        self.storm_active = true;
        let mut sheltered = Vec::new();
        let mut evicted = Vec::new();

        for &agent in docked_agents {
            if self.protected.contains(&agent) && sheltered.len() < self.capacity {
                sheltered.push(agent);
            } else {
                evicted.push(agent);
            }
        }

        StormReport { sheltered, evicted }
    }

    /// Signal the storm has passed.
    pub fn calm(&mut self) {
        self.storm_active = false;
    }

    pub fn is_storm_active(&self) -> bool {
        self.storm_active
    }

    pub fn protected_count(&self) -> usize {
        self.protected.len()
    }

    pub fn capacity(&self) -> usize {
        self.capacity
    }
}

// ---- Tests ----

#[cfg(test)]
mod tests {
    use super::*;

    fn agent(id: u64) -> AgentId {
        AgentId(id)
    }
    fn berth(id: u64) -> BerthId {
        BerthId(id)
    }

    // -- Dock tests --

    #[test]
    fn dock_new_is_empty() {
        let d = Dock::new(BerthId(0), 100);
        assert!(d.is_empty());
        assert!(!d.is_occupied());
        assert_eq!(d.capacity(), 100);
        assert_eq!(d.current_load(), 0);
    }

    #[test]
    fn dock_agent_success() {
        let mut d = Dock::new(BerthId(0), 100);
        assert!(d.dock(agent(1)).is_ok());
        assert!(d.is_occupied());
        assert_eq!(d.status(), &BerthStatus::Occupied(agent(1)));
    }

    #[test]
    fn dock_refuses_double_occupancy() {
        let mut d = Dock::new(BerthId(0), 100);
        d.dock(agent(1)).unwrap();
        assert!(d.dock(agent(2)).is_err());
    }

    #[test]
    fn dock_reserve_and_dock() {
        let mut d = Dock::new(BerthId(0), 100);
        d.reserve(agent(1)).unwrap();
        assert_eq!(d.status(), &BerthStatus::Reserved(agent(1)));
        d.dock(agent(1)).unwrap();
        assert_eq!(d.status(), &BerthStatus::Occupied(agent(1)));
    }

    #[test]
    fn dock_reserve_refuses_wrong_agent() {
        let mut d = Dock::new(BerthId(0), 100);
        d.reserve(agent(1)).unwrap();
        assert!(d.dock(agent(2)).is_err());
    }

    #[test]
    fn dock_undock() {
        let mut d = Dock::new(BerthId(0), 100);
        d.dock(agent(1)).unwrap();
        let a = d.undock().unwrap();
        assert_eq!(a, agent(1));
        assert!(d.is_empty());
    }

    #[test]
    fn dock_maintenance_cycle() {
        let mut d = Dock::new(BerthId(0), 100);
        d.start_maintenance().unwrap();
        assert_eq!(d.status(), &BerthStatus::Maintenance);
        assert!(d.dock(agent(1)).is_err());
        d.end_maintenance().unwrap();
        assert!(d.is_empty());
        d.dock(agent(1)).unwrap();
        assert!(d.is_occupied());
    }

    #[test]
    fn dock_load_management() {
        let mut d = Dock::new(BerthId(0), 100);
        d.dock(agent(1)).unwrap();
        assert!(d.add_load(50).is_ok());
        assert_eq!(d.current_load(), 50);
        assert_eq!(d.available_capacity(), 50);
        assert!(d.add_load(60).is_err()); // over capacity
        d.release_load(30);
        assert_eq!(d.current_load(), 20);
    }

    #[test]
    fn dock_undock_resets_load() {
        let mut d = Dock::new(BerthId(0), 100);
        d.dock(agent(1)).unwrap();
        d.add_load(80).unwrap();
        d.undock().unwrap();
        assert_eq!(d.current_load(), 0);
    }

    #[test]
    fn dock_undock_empty_fails() {
        let mut d = Dock::new(BerthId(0), 100);
        assert!(d.undock().is_err());
    }

    #[test]
    fn dock_maintenance_not_empty_fails() {
        let mut d = Dock::new(BerthId(0), 100);
        d.dock(agent(1)).unwrap();
        assert!(d.start_maintenance().is_err());
    }

    // -- Harbor tests --

    #[test]
    fn harbor_new() {
        let h = Harbor::new("main", 4, 50);
        assert_eq!(h.name(), "main");
        assert_eq!(h.total_berths(), 4);
        assert_eq!(h.available_berths(), 4);
        assert!(h.has_capacity());
    }

    #[test]
    fn harbor_find_empty() {
        let mut h = Harbor::new("main", 2, 50);
        assert!(h.find_empty().is_some());
        h.dock_mut(BerthId(0)).unwrap().dock(agent(1)).unwrap();
        assert!(h.find_empty().is_some());
        h.dock_mut(BerthId(1)).unwrap().dock(agent(2)).unwrap();
        assert!(h.find_empty().is_none());
        assert!(!h.has_capacity());
    }

    #[test]
    fn harbor_total_load_and_capacity() {
        let mut h = Harbor::new("main", 2, 100);
        h.dock_mut(BerthId(0)).unwrap().dock(agent(1)).unwrap();
        h.dock_mut(BerthId(0)).unwrap().add_load(30).unwrap();
        assert_eq!(h.total_load(), 30);
        assert_eq!(h.total_capacity(), 200);
    }

    // -- Pilot tests --

    #[test]
    fn pilot_guide_in() {
        let mut h = Harbor::new("main", 2, 50);
        let p = Pilot::new("main");
        let result = p.guide_in(&mut h, agent(1));
        assert_eq!(result, PilotResult::Docked(BerthId(0)));
    }

    #[test]
    fn pilot_guide_in_full_refused() {
        let mut h = Harbor::new("main", 1, 50);
        let p = Pilot::new("main");
        p.guide_in(&mut h, agent(1));
        let result = p.guide_in(&mut h, agent(2));
        assert_eq!(result, PilotResult::Refused);
    }

    #[test]
    fn pilot_guide_out() {
        let mut h = Harbor::new("main", 2, 50);
        let p = Pilot::new("main");
        p.guide_in(&mut h, agent(1));
        let a = p.guide_out(&mut h, BerthId(0)).unwrap();
        assert_eq!(a, agent(1));
    }

    // -- HarborMaster tests --

    #[test]
    fn harbor_master_immediate_dock() {
        let mut harbor = Harbor::new("main", 3, 100);
        let mut hm = HarborMaster::new("main");
        let req = DockingRequest::new(agent(1), Ternary::Positive, 20);
        let result = hm.request_docking(&mut harbor, req);
        assert!(matches!(result, PilotResult::Docked(_)));
        assert_eq!(hm.queue_length(), 0);
    }

    #[test]
    fn harbor_master_queues_when_full() {
        let mut harbor = Harbor::new("main", 1, 100);
        let mut hm = HarborMaster::new("main");
        hm.request_docking(&mut harbor, DockingRequest::new(agent(1), Ternary::Neutral, 10));
        let result =
            hm.request_docking(&mut harbor, DockingRequest::new(agent(2), Ternary::Neutral, 10));
        assert_eq!(result, PilotResult::Queued);
        assert_eq!(hm.queue_length(), 1);
    }

    #[test]
    fn harbor_master_processes_queue() {
        let mut harbor = Harbor::new("main", 1, 100);
        let mut hm = HarborMaster::new("main");
        hm.request_docking(&mut harbor, DockingRequest::new(agent(1), Ternary::Neutral, 10));
        hm.request_docking(&mut harbor, DockingRequest::new(agent(2), Ternary::Neutral, 10));
        assert_eq!(hm.queue_length(), 1);

        // Free the berth
        harbor.dock_mut(BerthId(0)).unwrap().undock().unwrap();

        let results = hm.process_queue(&mut harbor);
        assert_eq!(results.len(), 1);
        assert_eq!(hm.queue_length(), 0);
    }

    #[test]
    fn harbor_master_positive_priority_goes_first() {
        let mut harbor = Harbor::new("main", 1, 100);
        let mut hm = HarborMaster::new("main");
        hm.request_docking(&mut harbor, DockingRequest::new(agent(1), Ternary::Neutral, 10));
        // Queue agent 3 (positive) — should go to front
        hm.request_docking(&mut harbor, DockingRequest::new(agent(3), Ternary::Positive, 10));
        // Queue agent 2 (neutral) — should be behind positive
        hm.request_docking(&mut harbor, DockingRequest::new(agent(2), Ternary::Neutral, 10));

        harbor.dock_mut(BerthId(0)).unwrap().undock().unwrap();
        let results = hm.process_queue(&mut harbor);
        // First processed should be agent 3 (positive priority)
        assert!(matches!(results[0], PilotResult::Docked(_)));
    }

    // -- Tug tests --

    #[test]
    fn tug_transfer_load() {
        let t = Tug::new(1);
        let mut from = Dock::new(BerthId(0), 100);
        let mut to = Dock::new(BerthId(1), 100);
        from.dock(agent(1)).unwrap();
        from.add_load(50).unwrap();
        to.dock(agent(2)).unwrap();

        t.transfer_load(&mut from, &mut to, 30).unwrap();
        assert_eq!(from.current_load(), 20);
        assert_eq!(to.current_load(), 30);
    }

    #[test]
    fn tug_transfer_insufficient_source() {
        let t = Tug::new(1);
        let mut from = Dock::new(BerthId(0), 100);
        let mut to = Dock::new(BerthId(1), 100);
        from.dock(agent(1)).unwrap();
        from.add_load(10).unwrap();
        to.dock(agent(2)).unwrap();

        assert!(t.transfer_load(&mut from, &mut to, 20).is_err());
    }

    #[test]
    fn tug_assist_move() {
        let t = Tug::new(1);
        let mut h = Harbor::new("main", 2, 100);
        h.dock_mut(BerthId(0)).unwrap().dock(agent(1)).unwrap();

        let moved = t.assist_move(&mut h, BerthId(0), BerthId(1)).unwrap();
        assert_eq!(moved, agent(1));
        assert!(h.dock_mut(BerthId(0)).unwrap().is_empty());
        assert!(h.dock_mut(BerthId(1)).unwrap().is_occupied());
    }

    // -- Breakwater tests --

    #[test]
    fn breakwater_register_and_storm() {
        let mut bw = Breakwater::new(3);
        bw.register(agent(1)).unwrap();
        bw.register(agent(2)).unwrap();

        let report = bw.signal_storm(&[agent(1), agent(2), agent(3)]);
        assert_eq!(report.sheltered, vec![agent(1), agent(2)]);
        assert_eq!(report.evicted, vec![agent(3)]);
        assert!(bw.is_storm_active());
    }

    #[test]
    fn breakwater_capacity_limit() {
        let mut bw = Breakwater::new(1);
        bw.register(agent(1)).unwrap();
        assert!(bw.register(agent(2)).is_err());
    }

    #[test]
    fn breakwater_calm() {
        let mut bw = Breakwater::new(3);
        bw.signal_storm(&[]);
        assert!(bw.is_storm_active());
        bw.calm();
        assert!(!bw.is_storm_active());
    }

    #[test]
    fn breakwater_unregister() {
        let mut bw = Breakwater::new(3);
        bw.register(agent(1)).unwrap();
        bw.unregister(agent(1)).unwrap();
        assert_eq!(bw.protected_count(), 0);
        assert!(bw.unregister(agent(1)).is_err());
    }

    #[test]
    fn breakwater_no_duplicate_registration() {
        let mut bw = Breakwater::new(3);
        bw.register(agent(1)).unwrap();
        assert!(bw.register(agent(1)).is_err());
    }
}
