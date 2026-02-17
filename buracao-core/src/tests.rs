// src/tests.rs
use crate::baralho::{Baralho, Carta, Naipe, Valor};
use crate::estado::EstadoJogo;
use std::collections::HashMap;

// Função auxiliar para tirar cartas específicas do baralho
fn extrair_carta(baralho: &mut Baralho, valor: Valor, naipe: Naipe) -> Carta {
    let index = baralho
        .cartas
        .iter()
        .position(|c| c.valor == valor && c.naipe == naipe)
        .expect(&format!(
            "Erro: Carta {:?} {:?} não encontrada!",
            valor, naipe
        ));
    baralho.cartas.remove(index)
}

pub fn gerar_jogo_teste_bateria() -> EstadoJogo {
    // 1. Cria o Universo de 108 cartas
    let mut baralho_master = Baralho::new();
    // NÃO embaralhe para manter previsibilidade no debug

    let mut maos = vec![Vec::new(); 4];
    let mut jogos_time_a = HashMap::new();
    let jogos_time_b = HashMap::new();
    let mut lixo = Vec::new();

    // --- CENÁRIO: MESA COM CANASTRAS REAIS ---
    let sequencia = vec![
        Valor::Quatro,
        Valor::Cinco,
        Valor::Seis,
        Valor::Sete,
        Valor::Oito,
        Valor::Nove,
        Valor::Dez,
        Valor::Valete,
        Valor::Dama,
        Valor::Rei,
        Valor::As,
    ];

    let mut real_copas = Vec::new();
    for v in &sequencia {
        real_copas.push(extrair_carta(&mut baralho_master, *v, Naipe::Copas));
    }

    let mut real_ouros = Vec::new();
    for v in &sequencia {
        real_ouros.push(extrair_carta(&mut baralho_master, *v, Naipe::Ouros));
    }

    jogos_time_a.insert(1, real_copas); // ID 1
    jogos_time_a.insert(2, real_ouros); // ID 2

    // --- CENÁRIO: JOGADOR 0 (TIME A) PRONTO PARA BATER ---
    // Apenas 3 cartas: 4, 5, 6 de Espadas.
    let mut mao_p0 = Vec::new();
    mao_p0.push(extrair_carta(
        &mut baralho_master,
        Valor::Quatro,
        Naipe::Espadas,
    ));
    mao_p0.push(extrair_carta(
        &mut baralho_master,
        Valor::Cinco,
        Naipe::Espadas,
    ));
    mao_p0.push(extrair_carta(
        &mut baralho_master,
        Valor::Seis,
        Naipe::Espadas,
    ));

    maos[0] = mao_p0;

    // --- OPONENTES (JOGADORES 1, 2, 3) ---
    // Damos apenas 5 cartas para eles, para você diferenciar visualmente se logar errado.
    for i in 1..4 {
        for _ in 0..5 {
            if let Some(c) = baralho_master.comprar() {
                maos[i].push(c);
            }
        }
    }

    // Lixo inicial
    if let Some(c) = baralho_master.comprar() {
        lixo.push(c);
    }

    // Metadados
    let qtd_monte = baralho_master.cartas.len() as u32;
    let qtd_lixo = lixo.len() as u32;
    let verso_topo = baralho_master.cartas.last().map(|c| c.verso);

    // 2. CONSTRUÇÃO MANUAL (AQUI ESTÁ A CORREÇÃO DO STACK OVERFLOW)
    // Não chamamos EstadoJogo::new() aqui!
    //
    EstadoJogo {
        baralho: baralho_master,
        maos,
        turno_atual: 0, // Vez do Jogador 0
        lixo,
        jogos_time_a,
        jogos_time_b,
        pontuacao_a: 1000,
        historico_pontos_a: vec![500, 500],
        pontuacao_b: 500,
        historico_pontos_b: vec![800, -300],
        rodada: 10,
        numero_partida: 1,
        tres_vermelhos_time_a: Vec::new(),
        tres_vermelhos_time_b: Vec::new(),
        pegou_lixo_nesta_rodada: false,
        partida_encerrada: false,
        proximo_id_jogo: 3, // IDs 1 e 2 já usados
        baralho_acabou_nesta_rodada: false,
        comprou_nesta_rodada: false,
        qtd_monte,
        qtd_lixo,
        verso_topo,
    }
}
