use buracao_core::baralho::Carta;
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

/// Mescla a mão local (ordenada pelo usuário) com a mão do servidor (autoridade de conteúdo).
pub fn reconciliar_mao(mao_local: &[Carta], mao_servidor: Vec<Carta>) -> Vec<Carta> {
    let mut nova_mao = Vec::new();
    let mut pendencias_do_servidor = mao_servidor.clone();

    // 1. Percorre a mão local atual para preservar a ordem
    for carta_local in mao_local {
        // Verifica se essa carta local ainda existe na visão do servidor
        // Usamos position para lidar corretamente com duplicatas (2 baralhos)
        if let Some(idx) = pendencias_do_servidor.iter().position(|c| c == carta_local) {
            nova_mao.push(carta_local.clone());
            // Removemos da lista de pendências para não adicionar de novo depois
            pendencias_do_servidor.remove(idx);
        }
    }

    // 2. Tudo que sobrou em 'pendencias_do_servidor' são cartas NOVAS (compradas)
    // Adicionamos elas ao final da mão
    nova_mao.extend(pendencias_do_servidor);

    nova_mao
}
