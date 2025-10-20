# Async Multiplayer System

## Overview
Async multiplayer enables turn-based play-by-email/internet gameplay, where players take turns at their own pace. Each player receives a session file, replays the battle up to their turn, plans their move, and sends the updated file to the next player. When all players submit their turns, the simulation advances and everyone watches the action unfold.

---

## Core Workflow

### Typical Game Flow

```
Turn 1:
1. Player A receives session file (initial state)
2. Player A plans their moves
3. Player A confirms moves → generates action file
4. Action file sent to Player B

5. Player B receives action file
6. Player B watches Turn 1 replay (Player A's moves)
7. Player B plans their moves for Turn 1
8. Player B confirms moves → updates action file
9. Action file sent to Game Server/All Players

10. All players receive complete Turn 1 actions
11. All players watch Turn 1 simulation simultaneously
12. Turn 2 begins...
```

### Simultaneous Turn Resolution

All players submit their turns → Server processes → Everyone watches together:
```
Player A: Plans movement, weapon targeting
Player B: Plans movement, weapon targeting
Player C: Plans movement, weapon targeting
   ↓ All submit to server
Server: Validates all actions, runs simulation
   ↓ Broadcasts results
All Players: Watch 10-second simulation together
```

---

## File Format

### Session File Structure

```
session_game_001_turn_5.tribal
├── manifest.json              # Session metadata
├── initial_state.json.gz      # Starting game state
├── events_0_to_4.json.gz      # All events up to current turn
├── actions_turn_5.json.gz     # Pending actions for turn 5
└── snapshots/                 # Optional snapshots for replay
    ├── snapshot_turn_0.json.gz
    └── snapshot_turn_4.json.gz
```

### Manifest
```json
{
  "session_id": "game_001",
  "scenario": "Asteroid Belt Ambush",
  "current_turn": 5,
  "total_turns_played": 4,

  "players": [
    {
      "player_id": "player_a",
      "faction": "Terran",
      "status": "active",
      "last_action_turn": 4
    },
    {
      "player_id": "player_b",
      "faction": "Alien",
      "status": "active",
      "last_action_turn": 4
    }
  ],

  "turn_order": ["player_a", "player_b"],
  "current_player": "player_a",

  "game_state": {
    "phase": "planning",
    "turn_deadline": null,
    "all_players_ready": false
  },

  "file_format": {
    "version": 1,
    "compression": "gzip",
    "encryption": null
  },

  "created_at": 1705334400.0,
  "last_modified": 1705334500.0
}
```

### Actions File
```json
{
  "turn": 5,
  "player_id": "player_a",
  "timestamp": 1705334500.0,

  "actions": [
    {
      "action_type": "MovementPlanned",
      "ship_id": 42,
      "movement_mode": "MoveAndTurn",
      "target_position": { "x": 150.0, "y": 25.0, "z": -30.0 },
      "target_rotation": { "x": 0.0, "y": 0.707, "z": 0.0, "w": 0.707 }
    },
    {
      "action_type": "WeaponTargeted",
      "ship_id": 42,
      "weapon_id": 0,
      "target_id": 17,
      "target_subsystem": "ImpulseEngine",
      "fire_time": 3.5
    },
    {
      "action_type": "MovementConfirmed",
      "ship_id": 42
    }
  ],

  "checksum": "a3f5d8e9c2b1...",
  "signature": null
}
```

---

## Rust Implementation

### Session Manager
```rust
pub struct AsyncSession {
    /// Session metadata
    pub session_id: String,
    pub scenario_name: String,
    pub players: Vec<PlayerInfo>,

    /// Current game state
    pub game_state: GameState,
    pub event_store: EventStore,
    pub snapshot_manager: SnapshotManager,

    /// Turn management
    pub current_turn: u32,
    pub turn_order: Vec<PlayerId>,
    pub pending_actions: HashMap<PlayerId, Vec<PlayerAction>>,

    /// Session state
    pub phase: SessionPhase,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct PlayerInfo {
    pub player_id: PlayerId,
    pub player_name: String,
    pub faction: Faction,
    pub status: PlayerStatus,
    pub last_action_turn: u32,
}

#[derive(Clone, Serialize, Deserialize)]
pub enum PlayerStatus {
    Active,
    Ready,       // Submitted turn, waiting for others
    Disconnected,
    Eliminated,
}

#[derive(Clone, Serialize, Deserialize)]
pub enum SessionPhase {
    Planning,      // Players are planning moves
    WaitingForAll, // Some players submitted, waiting for rest
    Simulating,    // All players submitted, running simulation
    Replaying,     // Players watching simulation
}

impl AsyncSession {
    /// Create new async session
    pub fn new(
        scenario_name: String,
        players: Vec<PlayerInfo>,
    ) -> Self {
        Self {
            session_id: Self::generate_session_id(),
            scenario_name,
            players,
            game_state: GameState::new(),
            event_store: EventStore::new(),
            snapshot_manager: SnapshotManager::new(5, 20),
            current_turn: 1,
            turn_order: players.iter().map(|p| p.player_id.clone()).collect(),
            pending_actions: HashMap::new(),
            phase: SessionPhase::Planning,
        }
    }

    /// Load session from file
    pub fn load(path: &str) -> Result<Self, Error> {
        // Extract compressed archive
        let temp_dir = Self::extract_session_file(path)?;

        // Load manifest
        let manifest: SessionManifest =
            serde_json::from_str(&std::fs::read_to_string(
                format!("{}/manifest.json", temp_dir)
            )?)?;

        // Load initial state
        let initial_state = GameState::load_compressed(
            &format!("{}/initial_state.json.gz", temp_dir)
        )?;

        // Load events
        let event_store = EventStore::load(
            &format!("{}/events_0_to_{}.json.gz", temp_dir, manifest.current_turn - 1)
        )?;

        // Load snapshots
        let snapshot_manager = SnapshotManager::load_from_disk(
            &format!("{}/snapshots", temp_dir)
        )?;

        // Load pending actions
        let pending_actions = Self::load_pending_actions(&temp_dir, manifest.current_turn)?;

        Ok(Self {
            session_id: manifest.session_id,
            scenario_name: manifest.scenario,
            players: manifest.players,
            game_state: initial_state,
            event_store,
            snapshot_manager,
            current_turn: manifest.current_turn,
            turn_order: manifest.turn_order,
            pending_actions,
            phase: manifest.game_state.phase,
        })
    }

    /// Save session to file
    pub fn save(&self, path: &str) -> Result<(), Error> {
        // Create temporary directory
        let temp_dir = format!("{}_temp", path);
        std::fs::create_dir_all(&temp_dir)?;

        // Save manifest
        let manifest = self.create_manifest();
        std::fs::write(
            format!("{}/manifest.json", temp_dir),
            serde_json::to_string_pretty(&manifest)?
        )?;

        // Save initial state
        self.game_state.save_compressed(
            &format!("{}/initial_state.json.gz", temp_dir)
        )?;

        // Save events
        self.event_store.save_compressed(
            &format!("{}/events_0_to_{}.json.gz", temp_dir, self.current_turn - 1)
        )?;

        // Save snapshots
        let snapshot_dir = format!("{}/snapshots", temp_dir);
        std::fs::create_dir_all(&snapshot_dir)?;
        self.snapshot_manager.save_to_disk(&snapshot_dir)?;

        // Save pending actions
        self.save_pending_actions(&temp_dir)?;

        // Compress into single file
        Self::compress_session_dir(&temp_dir, path)?;

        // Cleanup temp directory
        std::fs::remove_dir_all(&temp_dir)?;

        Ok(())
    }

    /// Player submits actions for current turn
    pub fn submit_player_actions(
        &mut self,
        player_id: &PlayerId,
        actions: Vec<PlayerAction>,
    ) -> Result<(), Error> {
        // Validate player can submit
        if !self.can_player_submit(player_id) {
            return Err(Error::InvalidPlayer);
        }

        // Validate actions
        for action in &actions {
            self.validate_action(player_id, action)?;
        }

        // Store actions
        self.pending_actions.insert(player_id.clone(), actions);

        // Update player status
        if let Some(player) = self.players.iter_mut()
            .find(|p| &p.player_id == player_id) {
            player.status = PlayerStatus::Ready;
            player.last_action_turn = self.current_turn;
        }

        // Check if all players submitted
        if self.all_players_ready() {
            self.phase = SessionPhase::Simulating;
        } else {
            self.phase = SessionPhase::WaitingForAll;
        }

        Ok(())
    }

    /// Check if all players have submitted
    fn all_players_ready(&self) -> bool {
        self.players.iter()
            .filter(|p| p.status == PlayerStatus::Active || p.status == PlayerStatus::Ready)
            .all(|p| p.status == PlayerStatus::Ready)
    }

    /// Simulate turn with all player actions
    pub fn simulate_turn(&mut self) -> Result<(), Error> {
        if self.phase != SessionPhase::Simulating {
            return Err(Error::InvalidPhase);
        }

        // Convert pending actions to events
        let mut events = Vec::new();
        for (player_id, actions) in &self.pending_actions {
            for action in actions {
                events.push(self.action_to_event(player_id, action)?);
            }
        }

        // Sort events by time (for determinism)
        events.sort_by(|a, b| {
            a.simulation_time
                .partial_cmp(&b.simulation_time)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // Apply events and simulate
        for event in events {
            self.event_store.append(event.clone());
            self.game_state.apply_event(&event);
        }

        // Run 10-second simulation
        self.game_state.simulate_turn();

        // Create snapshot
        self.snapshot_manager.create_snapshot(&self.game_state);

        // Clear pending actions
        self.pending_actions.clear();

        // Reset player statuses
        for player in &mut self.players {
            if player.status == PlayerStatus::Ready {
                player.status = PlayerStatus::Active;
            }
        }

        // Advance turn
        self.current_turn += 1;
        self.phase = SessionPhase::Planning;

        Ok(())
    }

    /// Validate player action
    fn validate_action(
        &self,
        player_id: &PlayerId,
        action: &PlayerAction,
    ) -> Result<(), Error> {
        match action {
            PlayerAction::MovementPlanned { ship_id, target_position, .. } => {
                // Verify ship belongs to player
                if !self.player_owns_ship(player_id, *ship_id) {
                    return Err(Error::UnauthorizedAction);
                }

                // Verify movement is valid (range check, etc.)
                self.game_state.validate_movement(*ship_id, *target_position)?;
            }

            PlayerAction::WeaponTargeted { ship_id, target_id, .. } => {
                // Verify ship belongs to player
                if !self.player_owns_ship(player_id, *ship_id) {
                    return Err(Error::UnauthorizedAction);
                }

                // Verify target is valid
                if !self.game_state.entity_exists(*target_id) {
                    return Err(Error::InvalidTarget);
                }
            }

            _ => {}
        }

        Ok(())
    }
}
```

### Player Actions
```rust
#[derive(Clone, Serialize, Deserialize)]
pub enum PlayerAction {
    MovementPlanned {
        ship_id: EntityId,
        movement_mode: MovementMode,
        target_position: DVec3,
        target_rotation: DQuat,
    },

    MovementConfirmed {
        ship_id: EntityId,
    },

    WeaponTargeted {
        ship_id: EntityId,
        weapon_id: u32,
        target_id: EntityId,
        target_subsystem: Option<SubsystemId>,
        fire_time: f32,
    },

    SubsystemTargeted {
        ship_id: EntityId,
        target_subsystem: SubsystemId,
    },

    MovementModeChanged {
        ship_id: EntityId,
        new_mode: MovementMode,
    },
}

impl AsyncSession {
    /// Convert player action to game event
    fn action_to_event(
        &self,
        player_id: &PlayerId,
        action: &PlayerAction,
    ) -> Result<GameEvent, Error> {
        let event_type = match action {
            PlayerAction::MovementPlanned {
                ship_id,
                movement_mode,
                target_position,
                target_rotation,
            } => {
                EventType::Movement(MovementEvent::MovementPlanned {
                    turn: self.current_turn,
                    ship_id: *ship_id,
                    movement_mode: *movement_mode,
                    target_position: *target_position,
                    target_rotation: *target_rotation,
                    timestamp: self.get_timestamp(),
                })
            }

            PlayerAction::WeaponTargeted {
                ship_id,
                weapon_id,
                target_id,
                fire_time,
                ..
            } => {
                EventType::Combat(CombatEvent::WeaponFired {
                    turn: self.current_turn,
                    attacker_id: *ship_id,
                    weapon_id: *weapon_id,
                    target_id: *target_id,
                    target_subsystem: None,
                    fire_time: *fire_time,
                    timestamp: self.get_timestamp(),
                })
            }

            _ => return Err(Error::UnsupportedAction),
        };

        Ok(GameEvent {
            id: self.event_store.next_event_id(),
            turn: self.current_turn,
            simulation_time: None,
            timestamp: self.get_timestamp(),
            event_type,
        })
    }
}
```

---

## Network Transport

### File Transfer Options

#### 1. Direct File Sharing (Simple)
```rust
pub trait FileTransport {
    /// Send session file to player
    fn send_file(&self, recipient: &PlayerId, file_path: &str) -> Result<(), Error>;

    /// Receive session file
    fn receive_file(&self) -> Result<String, Error>;
}

// Email implementation
pub struct EmailTransport {
    smtp_server: String,
    sender_email: String,
}

impl FileTransport for EmailTransport {
    fn send_file(&self, recipient: &PlayerId, file_path: &str) -> Result<(), Error> {
        // Attach session file to email
        // Send via SMTP
        todo!()
    }

    fn receive_file(&self) -> Result<String, Error> {
        // Check inbox for session files
        // Download attachment
        todo!()
    }
}

// Cloud storage implementation
pub struct CloudStorageTransport {
    storage_provider: StorageProvider,
    bucket: String,
}

impl FileTransport for CloudStorageTransport {
    fn send_file(&self, recipient: &PlayerId, file_path: &str) -> Result<(), Error> {
        // Upload to cloud storage
        // Generate shareable link
        // Notify recipient
        todo!()
    }

    fn receive_file(&self) -> Result<String, Error> {
        // Download from cloud storage
        todo!()
    }
}
```

#### 2. Dedicated Server (Advanced)
```rust
pub struct AsyncGameServer {
    active_sessions: HashMap<String, AsyncSession>,
    player_connections: HashMap<PlayerId, PlayerConnection>,
}

impl AsyncGameServer {
    /// Player joins session
    pub async fn join_session(
        &mut self,
        player_id: PlayerId,
        session_id: String,
    ) -> Result<AsyncSession, Error> {
        let session = self.active_sessions
            .get(&session_id)
            .ok_or(Error::SessionNotFound)?;

        // Verify player is part of session
        if !session.players.iter().any(|p| p.player_id == player_id) {
            return Err(Error::UnauthorizedAccess);
        }

        Ok(session.clone())
    }

    /// Player submits turn
    pub async fn submit_turn(
        &mut self,
        player_id: PlayerId,
        session_id: String,
        actions: Vec<PlayerAction>,
    ) -> Result<(), Error> {
        let session = self.active_sessions
            .get_mut(&session_id)
            .ok_or(Error::SessionNotFound)?;

        session.submit_player_actions(&player_id, actions)?;

        // If all players ready, simulate turn
        if session.all_players_ready() {
            session.simulate_turn()?;

            // Notify all players
            self.broadcast_turn_complete(&session_id).await?;
        }

        Ok(())
    }

    /// Broadcast turn completion to all players
    async fn broadcast_turn_complete(&self, session_id: &str) -> Result<(), Error> {
        let session = self.active_sessions
            .get(session_id)
            .ok_or(Error::SessionNotFound)?;

        for player in &session.players {
            if let Some(connection) = self.player_connections.get(&player.player_id) {
                connection.notify_turn_complete(session_id).await?;
            }
        }

        Ok(())
    }
}
```

---

## Storage Solutions

### Cloud Storage Options

#### Amazon S3
```rust
use rusoto_s3::{S3Client, PutObjectRequest};

pub struct S3Storage {
    client: S3Client,
    bucket: String,
}

impl S3Storage {
    pub async fn upload_session(
        &self,
        session_id: &str,
        file_path: &str,
    ) -> Result<String, Error> {
        let body = std::fs::read(file_path)?;

        let request = PutObjectRequest {
            bucket: self.bucket.clone(),
            key: format!("sessions/{}.tribal", session_id),
            body: Some(body.into()),
            ..Default::default()
        };

        self.client.put_object(request).await?;

        // Generate presigned URL for download
        let url = self.generate_presigned_url(session_id)?;

        Ok(url)
    }

    pub async fn download_session(
        &self,
        session_id: &str,
        output_path: &str,
    ) -> Result<(), Error> {
        let request = GetObjectRequest {
            bucket: self.bucket.clone(),
            key: format!("sessions/{}.tribal", session_id),
            ..Default::default()
        };

        let result = self.client.get_object(request).await?;

        // Write to file
        let mut file = std::fs::File::create(output_path)?;
        let mut body = result.body.unwrap().into_async_read();
        tokio::io::copy(&mut body, &mut file).await?;

        Ok(())
    }
}
```

#### Azure Blob Storage
```rust
use azure_storage_blobs::prelude::*;

pub struct AzureBlobStorage {
    container_client: ContainerClient,
}

impl AzureBlobStorage {
    pub async fn upload_session(
        &self,
        session_id: &str,
        file_path: &str,
    ) -> Result<String, Error> {
        let blob_client = self.container_client
            .blob_client(format!("sessions/{}.tribal", session_id));

        let data = std::fs::read(file_path)?;
        blob_client.put_block_blob(data).await?;

        let url = blob_client.url()?;
        Ok(url.to_string())
    }
}
```

### Local Network Sharing
```rust
pub struct LocalNetworkShare {
    share_path: PathBuf,
}

impl LocalNetworkShare {
    pub fn upload_session(
        &self,
        session_id: &str,
        file_path: &str,
    ) -> Result<(), Error> {
        let dest = self.share_path
            .join(format!("{}.tribal", session_id));

        std::fs::copy(file_path, dest)?;
        Ok(())
    }

    pub fn download_session(
        &self,
        session_id: &str,
        output_path: &str,
    ) -> Result<(), Error> {
        let src = self.share_path
            .join(format!("{}.tribal", session_id));

        std::fs::copy(src, output_path)?;
        Ok(())
    }
}
```

---

## Anti-Cheat & Security

### Action Validation
```rust
impl AsyncSession {
    /// Verify action is legal and hasn't been tampered with
    fn validate_action_security(
        &self,
        player_id: &PlayerId,
        action: &PlayerAction,
        checksum: &str,
    ) -> Result<(), Error> {
        // 1. Verify checksum
        let calculated_checksum = self.calculate_action_checksum(action);
        if calculated_checksum != checksum {
            return Err(Error::ChecksumMismatch);
        }

        // 2. Verify action is within game rules
        self.validate_action(player_id, action)?;

        // 3. Verify action hasn't been submitted before
        if self.action_already_submitted(player_id, action) {
            return Err(Error::DuplicateAction);
        }

        Ok(())
    }

    fn calculate_action_checksum(&self, action: &PlayerAction) -> String {
        use sha2::{Sha256, Digest};

        let json = serde_json::to_string(action).unwrap();
        let mut hasher = Sha256::new();
        hasher.update(json.as_bytes());

        format!("{:x}", hasher.finalize())
    }
}
```

### Session Encryption (Optional)
```rust
use aes_gcm::{Aes256Gcm, Key, Nonce};
use aes_gcm::aead::{Aead, NewAead};

pub struct EncryptedSession {
    session: AsyncSession,
    encryption_key: Key,
}

impl EncryptedSession {
    pub fn save_encrypted(&self, path: &str) -> Result<(), Error> {
        // Save to temporary file
        let temp_path = format!("{}.tmp", path);
        self.session.save(&temp_path)?;

        // Read unencrypted data
        let plaintext = std::fs::read(&temp_path)?;

        // Encrypt
        let cipher = Aes256Gcm::new(&self.encryption_key);
        let nonce = Nonce::from_slice(b"unique nonce");
        let ciphertext = cipher.encrypt(nonce, plaintext.as_ref())
            .map_err(|_| Error::EncryptionFailed)?;

        // Write encrypted data
        std::fs::write(path, ciphertext)?;

        // Cleanup temp file
        std::fs::remove_file(&temp_path)?;

        Ok(())
    }

    pub fn load_encrypted(path: &str, key: &Key) -> Result<Self, Error> {
        // Read encrypted data
        let ciphertext = std::fs::read(path)?;

        // Decrypt
        let cipher = Aes256Gcm::new(key);
        let nonce = Nonce::from_slice(b"unique nonce");
        let plaintext = cipher.decrypt(nonce, ciphertext.as_ref())
            .map_err(|_| Error::DecryptionFailed)?;

        // Write to temp file
        let temp_path = format!("{}.tmp", path);
        std::fs::write(&temp_path, plaintext)?;

        // Load session
        let session = AsyncSession::load(&temp_path)?;

        // Cleanup temp file
        std::fs::remove_file(&temp_path)?;

        Ok(Self {
            session,
            encryption_key: key.clone(),
        })
    }
}
```

---

## UI Integration

### Async Multiplayer Menu
```rust
pub struct AsyncMultiplayerUI {
    active_sessions: Vec<SessionInfo>,
    selected_session: Option<String>,
}

impl AsyncMultiplayerUI {
    pub fn render(&mut self, ui: &Ui) {
        ui.window("Async Multiplayer")
            .size([600.0, 400.0], imgui::Condition::FirstUseEver)
            .build(|| {
                // Session list
                ui.text("Your Games:");

                for session in &self.active_sessions {
                    let status = match session.waiting_for {
                        Some(ref player_id) if player_id == &session.local_player_id => {
                            "Your Turn!"
                        }
                        Some(_) => "Waiting for opponent",
                        None => "Simulating...",
                    };

                    if ui.selectable(
                        &format!("{} - Turn {} - {}", session.name, session.current_turn, status)
                    ) {
                        self.selected_session = Some(session.session_id.clone());
                    }
                }

                ui.separator();

                // Actions
                if ui.button("New Game") {
                    self.create_new_game();
                }

                ui.same_line();

                if ui.button("Join Game") {
                    self.show_join_dialog();
                }

                ui.same_line();

                if ui.button("Load Game") {
                    if let Some(session_id) = &self.selected_session {
                        self.load_session(session_id);
                    }
                }
            });
    }

    fn load_session(&mut self, session_id: &str) {
        // Load session file
        // Enter game with replay of previous turns
        // Allow player to plan their turn
    }
}
```

---

## Future Enhancements

- **Mobile Support**: Play turns on mobile devices
- **Push Notifications**: Alert players when it's their turn
- **Matchmaking**: Find opponents for async games
- **Tournaments**: Organize async tournaments with brackets
- **Spectator Mode**: Watch ongoing async games
- **Turn Timer**: Optional time limits per turn
- **Auto-Submit**: Auto-submit if player doesn't act in time
- **Turn Reminders**: Email/SMS reminders for pending turns
- **Session Recovery**: Recover from corrupted/lost files
- **Bandwidth Optimization**: Delta compression for large sessions
- **Replay Highlights**: Auto-generate highlights from completed games
- **League System**: Ranked async multiplayer with ELO ratings
