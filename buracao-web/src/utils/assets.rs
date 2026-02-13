// Caminho base configurado no Trunk ou servidor de arquivos

#[derive(Clone, Copy)]
pub enum CardTheme {
    Paper,
    // Future: PixelArt, Modern, etc.
}

impl CardTheme {
    pub fn folder(&self) -> &str {
        match self {
            CardTheme::Paper => "/assets/cards/PaperCards",
        }
    }
}

/// Gera o caminho para a carta baseada no ID e no TEMA fornecido.
pub fn get_card_path(nome: &str, tema: &str) -> String {
    // Se o tema não termina com /, a gente adiciona
    if tema.ends_with('/') {
        format!("{}{}.png", tema, nome)
    } else {
        format!("{}/{}.png", tema, nome) // <--- Garante a barra aqui
    }
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
