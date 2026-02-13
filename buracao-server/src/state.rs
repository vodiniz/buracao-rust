use buracao_core::estado::EstadoJogo;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{RwLock, mpsc};
use warp::ws::Message;

// Tipos
pub type Sender = mpsc::UnboundedSender<Message>;
pub type RoomCode = String;
pub type DeviceId = String;
pub type PlayerId = u32;

pub struct Room {
    pub game_state: EstadoJogo,
    pub clients: HashMap<PlayerId, Sender>,
    pub sessions: HashMap<DeviceId, PlayerId>,
    pub player_names: HashMap<PlayerId, String>,
}

impl Room {
    pub fn new() -> Self {
        Self {
            game_state: EstadoJogo::new(),
            clients: HashMap::new(),
            sessions: HashMap::new(),
            player_names: HashMap::new(), // Inicializa vazio
        }
    }
}
// --- ESTADO GLOBAL DO SERVIDOR ---
pub struct ServerState {
    // Salas Ativas: "SALA-1" -> Dados da Sala
    // Usamos RwLock individual na Sala para que ações na Sala A não bloqueiem a Sala B
    pub rooms: HashMap<RoomCode, Arc<RwLock<Room>>>,
}

impl ServerState {
    pub fn new() -> Self {
        Self {
            rooms: HashMap::new(),
        }
    }
}

// O tipo que será passado para o Warp
pub type GlobalState = Arc<RwLock<ServerState>>;

pub fn inicializar_servidor() -> GlobalState {
    Arc::new(RwLock::new(ServerState::new()))
}
