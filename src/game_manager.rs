/// Game Manager - Tracks scenario parameters and game state
///
/// Manages turn-based gameplay, victory conditions, and scenario parameters

use serde::{Deserialize, Serialize};

/// Game mode - Edit or Play
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GameMode {
    /// Edit mode - place objects, configure scene
    Edit,
    /// Play mode - gameplay active
    Play,
}

impl Default for GameMode {
    fn default() -> Self {
        GameMode::Edit
    }
}

/// Pause state during play mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PauseState {
    #[default]
    Running,
    Paused,
}

/// Game Manager - Singleton managing game state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameManager {
    /// Current game mode (Edit or Play)
    #[serde(skip)]
    pub mode: GameMode,

    /// Pause state (only relevant in Play mode)
    #[serde(skip)]
    pub pause_state: PauseState,

    /// Current turn number
    pub current_turn: u32,

    /// Maximum turns (0 = unlimited)
    pub max_turns: u32,

    /// Scenario name
    pub scenario_name: String,

    /// Scenario description
    pub scenario_description: String,

    /// Victory conditions
    pub victory_conditions: VictoryConditions,

    /// Player faction
    pub player_faction: String,

    /// AI factions
    pub ai_factions: Vec<String>,

    /// Turn time limit in seconds (0 = no limit)
    pub turn_time_limit: f32,

    /// Game started timestamp
    #[serde(skip)]
    pub game_start_time: f32,

    /// Current turn start timestamp
    #[serde(skip)]
    pub turn_start_time: f32,
}

/// Victory condition types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VictoryConditions {
    /// Eliminate all enemy ships
    pub eliminate_all_enemies: bool,

    /// Survive for N turns
    pub survive_turns: u32,

    /// Destroy specific target
    pub destroy_target: Option<String>,

    /// Capture specific location
    pub capture_location: Option<String>,

    /// Custom win condition description
    pub custom_condition: Option<String>,
}

impl Default for VictoryConditions {
    fn default() -> Self {
        Self {
            eliminate_all_enemies: true,
            survive_turns: 0,
            destroy_target: None,
            capture_location: None,
            custom_condition: None,
        }
    }
}

impl Default for GameManager {
    fn default() -> Self {
        Self {
            mode: GameMode::Edit,
            pause_state: PauseState::Running,
            current_turn: 0,
            max_turns: 0,
            scenario_name: "Untitled Scenario".to_string(),
            scenario_description: "A space tactics scenario".to_string(),
            victory_conditions: VictoryConditions::default(),
            player_faction: "Player".to_string(),
            ai_factions: vec!["Red Team".to_string(), "Blue Team".to_string()],
            turn_time_limit: 0.0,
            game_start_time: 0.0,
            turn_start_time: 0.0,
        }
    }
}

impl GameManager {
    /// Create a new game manager
    pub fn new() -> Self {
        Self::default()
    }

    /// Start play mode - initialize game state
    pub fn start_play_mode(&mut self, current_time: f32) {
        self.mode = GameMode::Play;
        self.pause_state = PauseState::Running;
        self.current_turn = 1;
        self.game_start_time = current_time;
        self.turn_start_time = current_time;
        println!("=== PLAY MODE STARTED ===");
        println!("Scenario: {}", self.scenario_name);
        println!("Turn 1 begins!");
    }

    /// Stop play mode - return to edit mode
    pub fn stop_play_mode(&mut self) {
        self.mode = GameMode::Edit;
        self.pause_state = PauseState::Running;
        println!("=== EDIT MODE - Game Stopped ===");
    }

    /// Toggle pause state
    pub fn toggle_pause(&mut self) {
        if self.mode == GameMode::Play {
            self.pause_state = match self.pause_state {
                PauseState::Running => {
                    println!("=== GAME PAUSED ===");
                    PauseState::Paused
                }
                PauseState::Paused => {
                    println!("=== GAME RESUMED ===");
                    PauseState::Running
                }
            };
        }
    }

    /// Check if game is paused
    pub fn is_paused(&self) -> bool {
        matches!(self.pause_state, PauseState::Paused)
    }

    /// Check if in play mode
    pub fn is_playing(&self) -> bool {
        self.mode == GameMode::Play
    }

    /// Check if in edit mode
    pub fn is_editing(&self) -> bool {
        self.mode == GameMode::Edit
    }

    /// Advance to next turn
    pub fn next_turn(&mut self, current_time: f32) {
        if self.mode == GameMode::Play && !self.is_paused() {
            self.current_turn += 1;
            self.turn_start_time = current_time;
            println!("=== Turn {} begins! ===", self.current_turn);

            // Check if max turns reached
            if self.max_turns > 0 && self.current_turn > self.max_turns {
                println!("Max turns reached!");
            }
        }
    }

    /// Get elapsed game time
    pub fn get_elapsed_time(&self, current_time: f32) -> f32 {
        if self.mode == GameMode::Play {
            current_time - self.game_start_time
        } else {
            0.0
        }
    }

    /// Get elapsed turn time
    pub fn get_turn_elapsed_time(&self, current_time: f32) -> f32 {
        if self.mode == GameMode::Play {
            current_time - self.turn_start_time
        } else {
            0.0
        }
    }

    /// Check if victory conditions are met
    pub fn check_victory(&self) -> Option<String> {
        // TODO: Implement victory checking logic
        // For now, just return None
        None
    }

    /// Check if defeat conditions are met
    pub fn check_defeat(&self) -> Option<String> {
        // TODO: Implement defeat checking logic
        None
    }
}
