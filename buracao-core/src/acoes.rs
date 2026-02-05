use crate::baralho::{Carta, Verso};
use serde::{Deserialize, Serialize}; // Atenção: Pode precisar de ajuste circular se Visao usar Estado

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "tipo", content = "dados")] // Gera JSON limpo: { "tipo": "Descartar", "dados": { ... } }
pub enum AcaoJogador {
    /// O jogador quer comprar uma carta do monte fechado.
    ComprarBaralho,

    /// O jogador quer comprar o lixo.
    /// Em regras estritas, ele precisa provar que pode pegar o lixo usando a carta do topo
    /// em novos jogos ou em jogos existentes.
    ComprarLixo {
        /// Novos jogos que serão baixados usando a carta do topo do lixo
        novos_jogos: Vec<Vec<Carta>>,
        /// Cartas a serem adicionadas em jogos já existentes (índice do jogo, cartas)
        cartas_em_jogos_existentes: Vec<(u32, Vec<Carta>)>,
    },

    /// O jogador quer baixar jogos da mão (sequências ou trincas).
    BaixarJogos {
        jogos: Vec<Vec<Carta>>,
    },

    /// O jogador quer adicionar cartas a um jogo que já está na mesa.
    Ajuntar {
        indice_jogo: u32,
        cartas: Vec<Carta>,
    },

    /// O jogador descarta uma carta para finalizar o turno.
    Descartar {
        carta: Carta,
    },

    Mensagem {
        texto: String,
    },
}

// --- O que o Servidor MANDA para o Cliente ---
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "tipo", content = "conteudo")]
pub enum MsgServidor {
    // Quando conecta
    BoasVindas {
        id_jogador: u32,
    },

    // Atualização completa do estado (enviado a cada ação)
    Estado(VisaoJogador),

    // Notificações rápidas ("Fulano bateu", "Baralho no fim")
    Notificacao(String),

    // Erro de validação ("Não pode descartar essa carta")
    Erro(String),

    // Fim da partida com placar final
    FimDeJogo {
        vencedor_time: u8, // 0 (A) ou 1 (B)
        pontos_a: i32,
        pontos_b: i32,
        motivo: String, // "Batida" ou "Baralho Esgotado"
    },
}

// --- A "Foto" do jogo filtrada para cada jogador ---
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct VisaoJogador {
    // 1. Dados Pessoais
    pub meu_id: u32,
    pub minha_mao: Vec<Carta>, // Cartas REAIS
    pub posso_jogar: bool,     // Se é meu turno

    // 2. Dados Públicos da Mesa
    pub mesa_time_a: Vec<DetalheJogo>,
    pub mesa_time_b: Vec<DetalheJogo>,
    pub tres_vermelho_time_a: Vec<Carta>,
    pub tres_vermelho_time_b: Vec<Carta>,
    pub lixo: Option<Carta>, // No buraco aberto vê tudo, no fechado só o topo (ajuste na lógica)

    // 3. Dados dos Oponentes (Anonimizados)
    // Índice 0 = Jogador 0, Índice 1 = Jogador 1...
    pub qtd_cartas_jogadores: Vec<usize>,

    // 4. Placar e Status
    pub pontuacao_a: i32,
    pub pontuacao_b: i32,
    pub turno_atual: u32,
    pub rodada: u32,

    // 5. Metadados do Baralho
    pub cartas_no_monte: usize,

    pub qtd_monte: u32,
    pub qtd_lixo: u32,

    pub verso_topo: Option<Verso>,
}

// Uma representação simplificada de um jogo na mesa para o frontend
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DetalheJogo {
    pub id: u32,
    pub cartas: Vec<Carta>,
}
