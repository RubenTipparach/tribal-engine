/// Movement event system for turn-based tactical gameplay
///
/// Records all movement-related actions for replay and async multiplayer

use glam::{DVec3, DQuat};
use hecs::Entity;
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

/// Movement events for recording player actions
#[derive(Clone, Serialize, Deserialize, Debug)]
pub enum MovementEvent {
    /// Player started planning movement for a ship
    MovementPlanningStarted {
        turn: u32,
        ship_id: u64, // Entity ID as u64 for serialization
        timestamp: f64,
    },

    /// Widget position was updated during planning
    WidgetPositionChanged {
        turn: u32,
        ship_id: u64,
        new_position: DVec3,
        timestamp: f64,
    },

    /// Widget rotation was updated during planning
    WidgetRotationChanged {
        turn: u32,
        ship_id: u64,
        new_rotation: DQuat,
        timestamp: f64,
    },

    /// Player confirmed movement for this turn
    MovementConfirmed {
        turn: u32,
        ship_id: u64,
        start_position: DVec3,
        end_position: DVec3,
        control_point: DVec3,
        last_velocity: DVec3,
        start_rotation: DQuat,
        end_rotation: DQuat,
        timestamp: f64,
    },

    /// Player cancelled/reset movement
    MovementCancelled {
        turn: u32,
        ship_id: u64,
        timestamp: f64,
    },
}

impl MovementEvent {
    /// Get the turn number for this event
    pub fn turn(&self) -> u32 {
        match self {
            MovementEvent::MovementPlanningStarted { turn, .. } => *turn,
            MovementEvent::WidgetPositionChanged { turn, .. } => *turn,
            MovementEvent::WidgetRotationChanged { turn, .. } => *turn,
            MovementEvent::MovementConfirmed { turn, .. } => *turn,
            MovementEvent::MovementCancelled { turn, .. } => *turn,
        }
    }

    /// Get the timestamp for this event
    pub fn timestamp(&self) -> f64 {
        match self {
            MovementEvent::MovementPlanningStarted { timestamp, .. } => *timestamp,
            MovementEvent::WidgetPositionChanged { timestamp, .. } => *timestamp,
            MovementEvent::WidgetRotationChanged { timestamp, .. } => *timestamp,
            MovementEvent::MovementConfirmed { timestamp, .. } => *timestamp,
            MovementEvent::MovementCancelled { timestamp, .. } => *timestamp,
        }
    }

    /// Get the ship ID for this event
    pub fn ship_id(&self) -> u64 {
        match self {
            MovementEvent::MovementPlanningStarted { ship_id, .. } => *ship_id,
            MovementEvent::WidgetPositionChanged { ship_id, .. } => *ship_id,
            MovementEvent::WidgetRotationChanged { ship_id, .. } => *ship_id,
            MovementEvent::MovementConfirmed { ship_id, .. } => *ship_id,
            MovementEvent::MovementCancelled { ship_id, .. } => *ship_id,
        }
    }
}

/// Records movement events for replay and async multiplayer
pub struct MovementEventRecorder {
    /// All recorded events
    events: Vec<MovementEvent>,

    /// Current turn number
    current_turn: u32,

    /// Last recorded timestamp to throttle events
    last_record_time: f64,

    /// Minimum time between position/rotation events (seconds)
    throttle_interval: f64,
}

impl MovementEventRecorder {
    pub fn new(current_turn: u32) -> Self {
        Self {
            events: Vec::new(),
            current_turn,
            last_record_time: 0.0,
            throttle_interval: 0.05, // 20 events per second max
        }
    }

    /// Get current timestamp in seconds since epoch
    fn get_timestamp() -> f64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs_f64()
    }

    /// Convert hecs::Entity to u64 for serialization
    fn entity_to_u64(entity: Entity) -> u64 {
        entity.to_bits().get()
    }

    /// Record movement planning started
    pub fn record_planning_started(&mut self, ship_id: Entity) {
        let event = MovementEvent::MovementPlanningStarted {
            turn: self.current_turn,
            ship_id: Self::entity_to_u64(ship_id),
            timestamp: Self::get_timestamp(),
        };

        self.events.push(event);
    }

    /// Record widget position change (throttled)
    pub fn record_widget_position_change(&mut self, ship_id: Entity, new_position: DVec3) {
        let now = Self::get_timestamp();

        // Throttle position updates
        if now - self.last_record_time < self.throttle_interval {
            return;
        }

        let event = MovementEvent::WidgetPositionChanged {
            turn: self.current_turn,
            ship_id: Self::entity_to_u64(ship_id),
            new_position,
            timestamp: now,
        };

        self.events.push(event);
        self.last_record_time = now;
    }

    /// Record widget rotation change (throttled)
    pub fn record_widget_rotation_change(&mut self, ship_id: Entity, new_rotation: DQuat) {
        let now = Self::get_timestamp();

        // Throttle rotation updates
        if now - self.last_record_time < self.throttle_interval {
            return;
        }

        let event = MovementEvent::WidgetRotationChanged {
            turn: self.current_turn,
            ship_id: Self::entity_to_u64(ship_id),
            new_rotation,
            timestamp: now,
        };

        self.events.push(event);
        self.last_record_time = now;
    }

    /// Record movement confirmed
    pub fn record_movement_confirmed(
        &mut self,
        ship_id: Entity,
        start_position: DVec3,
        end_position: DVec3,
        control_point: DVec3,
        last_velocity: DVec3,
        start_rotation: DQuat,
        end_rotation: DQuat,
    ) {
        let event = MovementEvent::MovementConfirmed {
            turn: self.current_turn,
            ship_id: Self::entity_to_u64(ship_id),
            start_position,
            end_position,
            control_point,
            last_velocity,
            start_rotation,
            end_rotation,
            timestamp: Self::get_timestamp(),
        };

        self.events.push(event);
    }

    /// Record movement cancelled
    pub fn record_movement_cancelled(&mut self, ship_id: Entity) {
        let event = MovementEvent::MovementCancelled {
            turn: self.current_turn,
            ship_id: Self::entity_to_u64(ship_id),
            timestamp: Self::get_timestamp(),
        };

        self.events.push(event);
    }

    /// Get all events for current turn
    pub fn get_turn_events(&self) -> Vec<&MovementEvent> {
        self.events
            .iter()
            .filter(|e| e.turn() == self.current_turn)
            .collect()
    }

    /// Get all events
    pub fn get_all_events(&self) -> &[MovementEvent] {
        &self.events
    }

    /// Advance to next turn
    pub fn next_turn(&mut self) {
        self.current_turn += 1;
        self.last_record_time = 0.0;
    }

    /// Save events to JSON file
    pub fn save_to_file(&self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let json = serde_json::to_string_pretty(&self.events)?;
        std::fs::write(path, json)?;
        Ok(())
    }

    /// Load events from JSON file
    pub fn load_from_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let json = std::fs::read_to_string(path)?;
        let events: Vec<MovementEvent> = serde_json::from_str(&json)?;

        // Find the latest turn number
        let current_turn = events.iter().map(|e| e.turn()).max().unwrap_or(0);

        Ok(Self {
            events,
            current_turn,
            last_record_time: 0.0,
            throttle_interval: 0.05,
        })
    }

    /// Clear all events
    pub fn clear(&mut self) {
        self.events.clear();
        self.current_turn = 0;
        self.last_record_time = 0.0;
    }

    /// Get event count
    pub fn event_count(&self) -> usize {
        self.events.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_recorder() {
        let mut recorder = MovementEventRecorder::new(1);

        // Create a dummy entity
        let entity = Entity::DANGLING;

        // Record some events
        recorder.record_planning_started(entity);
        recorder.record_widget_position_change(entity, DVec3::new(10.0, 0.0, 0.0));
        recorder.record_movement_confirmed(
            entity,
            DVec3::ZERO,
            DVec3::new(10.0, 0.0, 0.0),
            DVec3::new(4.0, 0.0, 0.0),
            DVec3::ZERO,
            DQuat::IDENTITY,
            DQuat::IDENTITY,
        );

        assert_eq!(recorder.event_count(), 3);

        let turn_events = recorder.get_turn_events();
        assert_eq!(turn_events.len(), 3);
    }

    #[test]
    fn test_save_load() {
        let mut recorder = MovementEventRecorder::new(1);
        let entity = Entity::DANGLING;

        recorder.record_planning_started(entity);

        // Save to temp file
        let temp_path = "test_events.json";
        recorder.save_to_file(temp_path).unwrap();

        // Load back
        let loaded = MovementEventRecorder::load_from_file(temp_path).unwrap();
        assert_eq!(loaded.event_count(), 1);

        // Cleanup
        std::fs::remove_file(temp_path).ok();
    }
}
