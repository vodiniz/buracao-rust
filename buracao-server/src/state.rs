use buracao_core::estado::EstadoJogo;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{RwLock, mpsc};
use warp::ws::Message;

// Tipos para facilitar leitura
pub type Sender = mpsc::UnboundedSender<Message>;
pub type RoomCode = String;
pub type DeviceId = String;
pub type PlayerId = u32;

// --- ESTRUTURA DE UMA SALA ---
pub struct Room {
    pub game_state: EstadoJogo,
    // Mapeia: ID do Jogador (0, 1, 2, 3) -> Canal de envio do WebSocket dele
    pub clients: HashMap<PlayerId, Sender>,
    // Mapeia: DeviceID (LocalStorage) -> ID do Jogador (0, 1, 2, 3)
    pub sessions: HashMap<DeviceId, PlayerId>,
}

impl Room {
    pub fn new() -> Self {
        Self {
            game_state: EstadoJogo::new(),
            clients: HashMap::new(),
            sessions: HashMap::new(),
        }
    }
}

// --- ESTADO GLOBAL DO SERVIDOR ---
pub struct ServerState {
    // Salas Ativas: "SALA-1" -> Dados da Sala
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
