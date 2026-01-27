use rand::seq::SliceRandom; // Import necessário para o shuffle
use serde::{Deserialize, Serialize};

// --- ESTRUTURA DAS CARTAS ---

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, PartialOrd, Ord)]
pub enum Naipe {
    Copas,
    Espadas,
    Ouros,
    Paus,
    // Para o Joker (Coringa do baralho), o naipe pode ser irrelevante ou "Nenhum"
    Nenhum,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, PartialOrd, Ord)]
pub enum Valor {
    Tres,
    Quatro,
    Cinco,
    Seis,
    Sete,
    Oito,
    Nove,
    Dez,
    Valete,
    Dama,
    Rei,
    As,
    Dois,  // Importante: No Buraco, o 2 é coringa!
    Joker, // O Coringão clássico
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, PartialOrd, Eq, Ord)]
pub struct Carta {
    pub naipe: Naipe,
    pub valor: Valor,
}

impl Carta {
    /// Retorna quantos pontos a carta vale na contagem final
    pub fn pontos(&self) -> i32 {
        match self.valor {
            Valor::Joker => 20, // Coringa vale 20 (varia da regra, ajuste se precisar)
            _ => 10,            // Outras cartas valem 5
        }
    }

    /// Verifica se a carta funciona como coringa (2 ou Joker)
    pub fn eh_coringa(&self) -> bool {
        matches!(self.valor, Valor::Dois | Valor::Joker)
    }

    pub fn valor_numerico_sequencia(&self) -> u8 {
        match self.valor {
            Valor::Quatro => 4,
            Valor::Cinco => 5,
            Valor::Seis => 6,
            Valor::Sete => 7,
            Valor::Oito => 8,
            Valor::Nove => 9,
            Valor::Dez => 10,
            Valor::Valete => 11,
            Valor::Dama => 12,
            Valor::Rei => 13,
            Valor::As => 14, // No seu jogo, A é a mais alta
            _ => 0,          // 2, 3 (se houver), e Joker são tratados como "especiais"
        }
    }
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct Baralho {
    pub cartas: Vec<Carta>,
}

impl Baralho {
    /// Cria um baralho novo com 2 jogos completos (Regra do Buraco)
    pub fn new() -> Self {
        let mut cartas = Vec::new();

        // Buraco usa 2 baralhos
        for _ in 0..2 {
            // Adiciona as cartas normais (A a K)
            for naipe in [Naipe::Copas, Naipe::Ouros, Naipe::Espadas, Naipe::Paus] {
                for valor in [
                    Valor::As,
                    Valor::Dois,
                    Valor::Tres,
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
                ] {
                    cartas.push(Carta { naipe, valor });
                }
            }
            // Adiciona 2 Jokers por baralho (geralmente)
            cartas.push(Carta {
                naipe: Naipe::Nenhum,
                valor: Valor::Joker,
            });
            cartas.push(Carta {
                naipe: Naipe::Nenhum,
                valor: Valor::Joker,
            });
        }

        Self { cartas }
    }

    /// Embaralha as cartas no lugar (in-place)
    pub fn shuffle(&mut self) {
        let mut rng = rand::rng();
        self.cartas.shuffle(&mut rng);
    }

    /// Tira a carta do topo do baralho (para comprar ou dar as cartas)
    pub fn comprar(&mut self) -> Option<Carta> {
        self.cartas.pop()
    }

    /// Verifica quantas cartas restam
    pub fn restantes(&self) -> usize {
        self.cartas.len()
    }
}
// --- PROTOCOLO DE COMUNICAÇÃO (JSON) ---

// O que o servidor manda para os jogadores
#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "tipo", content = "dados")] // Isso cria um JSON bonito: { "tipo": "Estado", "dados": {...} }
pub enum MsgServidor {
    BoasVindas { id_jogador: u8 },
    EstadoDoJogo(EstadoJogo),
    Erro(String),
}

// O que o jogador manda para o servidor
#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "acao", content = "detalhes")]
pub enum MsgCliente {
    ComprarBaralho,
    ComprarLixo,
    // Exemplo: Baixar ([As, 2, 3], grupos de cartas)
    BaixarJogos { jogos: Vec<Vec<Carta>> },
    Descartar { carta: Carta },
}

// --- ESTADO GLOBAL DO JOGO ---

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct EstadoJogo {
    pub baralho: Baralho,
    pub maos: Vec<Vec<Carta>>,
    pub turno_atual: usize, // 0 a 3
    pub lixo: Vec<Carta>,
    pub jogos_time_a: Vec<Vec<Carta>>, // Canastras baixadas time A
    pub jogos_time_b: Vec<Vec<Carta>>, // Canastras baixadas time B
    pub pontuacao_a: i32,
    pub pontuacao_b: i32,
    pub jogador_pe: usize,
    pub tres_time_a: Vec<Carta>,
    pub tres_time_b: Vec<Carta>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct VisaoJogador {
    // A mão dele é a única que ele vê de verdade
    pub minha_mao: Vec<Carta>,

    // Dos outros, ele só precisa saber QUANTAS cartas têm (para desenhar o verso da carta na tela)
    // Ex: {0: 11, 1: 10, 2: 11, 3: 9}
    pub qtd_cartas_jogadores: Vec<usize>,

    pub lixo: Vec<Carta>, // O lixo é público (ou só o topo, dependendo da regra)
    pub jogos_mesa: Vec<Vec<Carta>>, // As canastras baixadas são públicas
    pub turno_atual: usize,
    pub cartas_no_monte: usize, // Só o número, nada de lista de cartas!
}

impl EstadoJogo {
    pub fn new() -> Self {
        Self {
            baralho: Baralho::new(),
            maos: vec![Vec::new(); 4],
            turno_atual: 0,
            lixo: Vec::new(),
            jogos_time_a: Vec::new(),
            jogos_time_b: Vec::new(),
            pontuacao_a: 0,
            pontuacao_b: 0,
            jogador_pe: 0,
            tres_time_a: Vec::new(),
            tres_time_b: Vec::new(),
        }
    }

    pub fn dar_cartas(&mut self) {
        self.baralho.shuffle();

        let mut maos: Vec<Vec<Carta>> = vec![Vec::new(); 4];
        for _ in 0..15 {
            for mao in &mut maos {
                if let Some(carta) = self.baralho.comprar() {
                    mao.push(carta);
                }
            }
        }
    }

    fn pontos_para_descer(&self, pontuação_time: i32) -> i32 {
        if pontuação_time < 2500 {
            80
        } else {
            100
        }
    }

    pub fn tentar_comprar_lixo(
        &mut self,
        jogador_id: usize,
        jogos_propostos: Vec<Vec<Carta>>,
    ) -> Result<(), String> {
        // --- 1. VALIDAÇÕES BÁSICAS ---

        if self.turno_atual != jogador_id {
            return Err("Não é seu turno".to_string());
        }

        let carta_topo_lixo = match self.lixo.last() {
            Some(c) => c.clone(),
            None => return Err("Lixo vazio".to_string()),
        };

        // --- 2. VERIFICAR O USO DO LIXO ---

        // Percorre todos os jogos propostos e vê se a carta do topo está em pelo menos um deles
        let usa_topo_lixo = jogos_propostos
            .iter()
            .any(|jogo| jogo.iter().any(|c| c == &carta_topo_lixo));

        if !usa_topo_lixo {
            return Err(
                "Você precisa utilizar a carta do topo do lixo em um dos jogos.".to_string(),
            );
        }

        // --- 3. VALIDAR SE TODOS OS JOGOS SÃO VÁLIDOS ---

        for (i, jogo) in jogos_propostos.iter().enumerate() {
            // Usa aquela função 'validar_sequencia' que criamos antes
            if !validar_sequencia(jogo) {
                return Err(format!(
                    "O jogo número {} enviado é inválido (não é sequência).",
                    i + 1
                ));
            }
        }

        // --- 4. CÁLCULO DE PONTOS (SOMATÓRIA GERAL) ---

        let time_id = jogador_id % 2;
        let jogos_do_time = if time_id == 0 {
            &self.jogos_time_a
        } else {
            &self.jogos_time_b
        };
        let ja_abriu = !jogos_do_time.is_empty();

        if !ja_abriu {
            let pontuacao_atual = if time_id == 0 {
                self.pontuacao_a
            } else {
                self.pontuacao_b
            };
            let minimo_necessario = self.pontos_para_descer(pontuacao_atual); // 30, 80 ou 100

            // Soma pontos de TODAS as cartas de TODOS os jogos propostos
            let total_pontos: i32 = jogos_propostos
                .iter()
                .map(|jogo| jogo.iter().map(|c| c.pontos()).sum::<i32>())
                .sum();

            if total_pontos < minimo_necessario {
                return Err(format!(
                    "Pontuação insuficiente. Necessário: {}, Somado: {}",
                    minimo_necessario, total_pontos
                ));
            }
        }

        // --- 5. EXECUÇÃO DA JOGADA (TRANSAÇÃO) ---

        // A. Move TODO o lixo para a mão do jogador primeiro.
        // Isso facilita a lógica de remoção depois (evita erro de "carta não encontrada" se ele usar a do lixo).
        let mut lixo_inteiro: Vec<Carta> = self.lixo.drain(..).collect();
        self.maos[jogador_id].append(&mut lixo_inteiro);

        // B. Remove as cartas usadas da mão e coloca na mesa
        for jogo in jogos_propostos {
            // Remove da mão
            for carta_jogo in &jogo {
                // Procura a carta na mão e remove a primeira ocorrência
                if let Some(pos) = self.maos[jogador_id].iter().position(|c| c == carta_jogo) {
                    self.maos[jogador_id].remove(pos);
                } else {
                    // Isso teoricamente nunca deve acontecer se o frontend e as validações estiverem certos,
                    // mas em Rust seguro vale a pena tratar ou dar panic controlado.
                    // Aqui, se não achar, significa que validamos errado antes.
                    eprintln!(
                        "ERRO CRÍTICO: Carta validada não encontrada na mão após pegar lixo!"
                    );
                }
            }

            // Adiciona na mesa
            if time_id == 0 {
                self.jogos_time_a.push(jogo);
            } else {
                self.jogos_time_b.push(jogo);
            }
        }

        // Opcional: Reordenar a mão do jogador após a bagunça
        self.maos[jogador_id].sort();

        Ok(())
    }

    /// Cria um recorte seguro do estado para um jogador específico
    pub fn gerar_visao_para(&self, jogador_alvo_id: usize) -> VisaoJogador {
        todo!();
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub enum AcaoJogador {
    // ... outras ações
    ComprarLixo {
        // As cartas QUE ESTÃO NA MÃO do jogador e serão usadas junto com a do topo
        cartas_para_descer: Vec<Carta>,
    },
}

pub fn validar_sequencia(cartas: &[Carta]) -> bool {
    // Regra 1: Mínimo de 3 cartas
    if cartas.len() < 3 {
        return false;
    }

    // --- PASSO A: SEPARAR CORINGAS DE NATURAIS ---
    let mut naturais: Vec<&Carta> = Vec::new();
    let mut coringas_count = 0;

    for c in cartas {
        if c.eh_coringa() {
            coringas_count += 1;
        } else {
            naturais.push(c);
        }
    }

    // Regra Buraco: Máximo 1 coringa por sequência
    if coringas_count > 1 {
        return false;
    }

    // --- PASSO B: VALIDAR NAIPE ---
    // Pega o naipe da primeira carta natural como referência
    let naipe_referencia = naturais[0].naipe;
    for c in &naturais {
        if c.naipe != naipe_referencia {
            return false; // Misturou naipes
        }
    }

    // --- PASSO C: ORDENAR E VERIFICAR LACUNAS ---
    // Ordena pelo valor numérico (4, 5, 6...)
    naturais.sort_by_key(|c| c.valor_numerico_sequencia());

    let mut lacunas_para_preencher = 0;

    // Percorre as cartas comparando a atual com a próxima
    for i in 0..(naturais.len() - 1) {
        let valor_atual = naturais[i].valor_numerico_sequencia();
        let valor_proximo = naturais[i + 1].valor_numerico_sequencia();

        // Checagem de segurança (se entrou carta inválida como 3 ou outro Joker disfarçado)
        if valor_atual == 0 || valor_proximo == 0 {
            return false;
        }

        // Se tiver duplicata (ex: 4, 4, 5), é inválido
        if valor_atual == valor_proximo {
            return false;
        }

        // Calcula o "salto".
        // Ex: 4 e 5 -> diff 1 (ok)
        // Ex: 4 e 6 -> diff 2 (falta 1 carta)
        // Ex: 4 e 7 -> diff 3 (faltam 2 cartas)
        let diff = valor_proximo - valor_atual - 1;

        // O numero de cartas faltando é (diff - 1)
        lacunas_para_preencher += diff;
    }

    // --- PASSO D: CONTABILIDADE FINAL ---

    // Temos coringas suficientes para tapar os buracos do meio?
    if coringas_count >= lacunas_para_preencher {
        // Sim! E se sobrar coringa?
        // Ex: Tenho [4, 5]. Lacunas = 0. Coringas = 1.
        // O coringa vira o 3 ou o 6. Isso é válido.
        true
    } else {
        // Não temos coringas suficientes para conectar os números
        // Ex: [4, 7] (precisa de 2 cartas para virar 4-5-6-7), mas só tenho 1 Joker.
        false
    }
}
