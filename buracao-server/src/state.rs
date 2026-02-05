use buracao_core::estado::EstadoJogo;
use dashmap::DashMap;
use std::sync::{Arc, atomic::AtomicUsize};
use tokio::sync::{RwLock, mpsc};

// Tipos públicos para serem usados no main e no handler
pub type JogoCompartilhado = Arc<RwLock<EstadoJogo>>;
pub type Clientes = Arc<DashMap<usize, mpsc::UnboundedSender<warp::ws::Message>>>;

// ID Global (Atomic para ser thread-safe)
pub static NEXT_USER_ID: AtomicUsize = AtomicUsize::new(1);

// Função auxiliar para inicializar o jogo
pub fn inicializar_jogo() -> JogoCompartilhado {
    let mut estado_inicial = EstadoJogo::new();
    println!("🎲 Embaralhando e distribuindo cartas...");
    estado_inicial.dar_cartas();
    Arc::new(RwLock::new(estado_inicial))
}

// Função auxiliar para inicializar a lista de clientes
pub fn inicializar_clientes() -> Clientes {
    Arc::new(DashMap::new())
}
