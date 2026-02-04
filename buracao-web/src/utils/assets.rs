// Caminho base configurado no Trunk ou servidor de arquivos
const CARDS_BASE_PATH: &str = "/assets/cards/PaperCards1.1";

#[derive(Clone, Copy)]
pub enum CardTheme {
    Paper,
    // Future: PixelArt, Modern, etc.
}

impl CardTheme {
    pub fn folder(&self) -> &str {
        match self {
            CardTheme::Paper => "/assets/cards/PaperCards1.1",
        }
    }
}

/// Gera o caminho para a carta baseada no ID e no TEMA fornecido.
pub fn get_card_path(card_id: &str, theme_folder: &str) -> String {
    // theme_folder ex: "assets/cards/PaperCards1.1"
    format!("{}/{}.png", theme_folder, card_id)
}

/// Gera o caminho para o verso (Back) baseado no TEMA.
pub fn get_back_path(color: &str, theme_folder: &str) -> String {
    format!("{}/back_{}.png", theme_folder, color)
}

// Mantém a função de cor igual, pois ela não depende de asset
pub fn get_selection_color(group_index: usize) -> &'static str {
    match group_index % 5 {
        0 => "blue",
        1 => "green",
        2 => "orange",
        3 => "purple",
        4 => "cyan",
        _ => "red",
    }
}
