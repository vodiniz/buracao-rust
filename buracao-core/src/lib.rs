// Conteúdo de lib.rs
pub mod acoes;
pub mod baralho;
pub mod estado;
pub mod regras;
pub mod tests;

// Facilita a vida de quem usa:
pub use acoes::AcaoJogador;
pub use baralho::{Carta, Naipe, Valor, Verso};
pub use estado::EstadoJogo;
