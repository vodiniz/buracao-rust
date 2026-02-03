/// Utilitário responsável por resolver os caminhos estáticos dos assets.
/// Centraliza a lógica de strings para evitar "magic strings" espalhadas pelos componentes.

// Caminho base configurado no Trunk ou servidor de arquivos
const CARDS_BASE_PATH: &str = "/assets/cards/PaperCards1.1";

/// Gera o caminho completo para uma carta baseada no identificador.
///
/// # Argumentos
/// * `card_id` - O identificador da carta conforme sua convenção (ex: "c_2", "d_q", "joker_b").
///
/// # Retorno
/// Uma `String` contendo o caminho relativo para o arquivo png.
pub fn get_card_path(card_id: &str) -> String {
    format!("{}/{}.png", CARDS_BASE_PATH, card_id)
}

/// Gera o caminho para o verso da carta (Back).
///
/// # Argumentos
/// * `color` - "b" para Blue (Azul), "r" para Red (Vermelho).
pub fn get_back_path(color: &str) -> String {
    format!("{}/back_{}.png", CARDS_BASE_PATH, color)
}

/// Retorna a cor CSS correspondente ao índice do grupo de seleção.
/// Usado para colorir a borda/fundo da carta quando selecionada para um jogo.
pub fn get_selection_color(group_index: usize) -> &'static str {
    match group_index % 5 {
        0 => "blue",   // Grupo 0: Azul
        1 => "green",  // Grupo 1: Verde
        2 => "orange", // Grupo 2: Laranja
        3 => "purple", // Grupo 3: Roxo
        4 => "cyan",   // Grupo 4: Ciano
        _ => "red",    // Fallback
    }
}
