// Retorna a posição relativa na tela ("bottom", "right", "top", "left")
pub fn get_relative_position(my_id: u32, target_id: u32) -> &'static str {
    // Supondo 4 jogadores. A matemática modular resolve a rotação.
    let diff = (target_id as i32 - my_id as i32 + 4) % 4;
    match diff {
        0 => "bottom", // Eu
        1 => "right",  // Jogador à minha direita
        2 => "top",    // Meu parceiro
        3 => "left",   // Jogador à minha esquerda
        _ => "unknown",
    }
}
