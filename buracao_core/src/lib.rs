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

    pub fn eh_tres_vermelho(&self) -> bool {
        self.valor == Valor::Tres && (self.naipe == Naipe::Copas || self.naipe == Naipe::Ouros)
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
    pub turno_atual: u32, // 0 a 3
    pub lixo: Vec<Carta>,
    pub jogos_time_a: Vec<Vec<Carta>>, // Canastras baixadas time A
    pub jogos_time_b: Vec<Vec<Carta>>, // Canastras baixadas time B
    pub pontuacao_a: i32,
    pub pontuacao_b: i32,
    pub rodada: u32,
    pub tres_vermelhos_time_a: Vec<Carta>,
    pub tres_vermelhos_time_b: Vec<Carta>,
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
            rodada: 0,
            tres_vermelhos_time_a: Vec::new(),
            tres_vermelhos_time_b: Vec::new(),
        }
    }

    pub fn dar_cartas(&mut self) {
        self.baralho.shuffle();

        let mut maos: Vec<Vec<Carta>> = vec![Vec::new(); 4];

        // REGRA BURACO: Geralmente são 11 cartas por jogador.
        // O código original tinha 15. Ajustei para 11, mas altere se for uma variante.
        for _ in 0..11 {
            for mao in &mut maos {
                if let Some(carta) = self.baralho.comprar() {
                    mao.push(carta);
                }
            }
        }

        self.maos = maos;

        // --- CORREÇÃO DO ERRO ---
        // 1. Usamos '0usize..4' para garantir que 'i' seja usize.
        // 2. Usamos 'self.rodada as usize' para converter o i32 em usize antes de somar.
        for i in 0usize..4 {
            let jogador_idx = (self.rodada as usize + i) % 4;
            self.processar_tres_vermelhos(jogador_idx);
        }
    }

    pub fn processar_tres_vermelhos(&mut self, jogador_id: usize) {
        // Loop para garantir recursividade (se comprar um 3, processa de novo)
        loop {
            // Acesso seguro: Drena a mão inteira para separar as cartas
            // Isso evita conflitos de borrow checker porque a mão fica vazia temporariamente
            let (novos_tres_vermelhos, resto_da_mao): (Vec<Carta>, Vec<Carta>) = self.maos
                [jogador_id]
                .drain(..)
                .partition(|c| c.eh_tres_vermelho());

            // 1. Devolve as cartas normais para a mão
            self.maos[jogador_id] = resto_da_mao;

            // 2. Se não achou nenhum 3 vermelho, o trabalho acabou. Sai do loop.
            if novos_tres_vermelhos.is_empty() {
                break;
            }

            // 3. Processa os 3 vermelhos encontrados
            let qtd_reposicao = novos_tres_vermelhos.len();

            let time_id = jogador_id % 2;
            if time_id == 0 {
                self.tres_vermelhos_time_a.extend(novos_tres_vermelhos);
            } else {
                self.tres_vermelhos_time_b.extend(novos_tres_vermelhos);
            }

            // 4. Compra novas cartas para repor
            for _ in 0..qtd_reposicao {
                if let Some(carta) = self.baralho.comprar() {
                    self.maos[jogador_id].push(carta);
                }
            }

            // O loop reinicia aqui.
            // Na próxima iteração, ele vai checar se as NOVAS cartas compradas
            // também são 3 vermelhos. Se não forem, o 'partition' retorna vazio
            // e cai no 'break'.
        }

        self.maos[jogador_id].sort();
    }

    fn pontos_para_descer(&self, id_jogador: u32) -> i32 {
        if id_jogador.is_multiple_of(2) {
            if self.pontuacao_a < 2500 {
                80
            } else {
                100
            }
        } else if self.pontuacao_b < 2500 {
            80
        } else {
            100
        }
    }

    pub fn conferir_real(&self, id_jogador: usize) -> bool {
        // 1. Seleciona o vetor do time correto (sem clonar!)
        let jogos_do_time = if id_jogador % 2 == 0 {
            &self.jogos_time_a
        } else {
            &self.jogos_time_b
        };

        // 2. Verifica se ALGUM jogo satisfaz a condição
        // Condição de Canastra: Tamanho >= 7
        jogos_do_time.iter().any(|jogo| jogo.len() >= 7)
    }

    pub fn obter_canastras(&self, id_jogador: usize) -> Vec<&Vec<Carta>> {
        let jogos_do_time = if id_jogador.is_multiple_of(2) {
            &self.jogos_time_a
        } else {
            &self.jogos_time_b
        };

        // Filtra e coleta referências
        let canastras: Vec<&Vec<Carta>> = jogos_do_time
            .iter()
            .filter(|jogo| jogo.len() >= 7)
            .collect();

        canastras
    }

    pub fn contar_pontos(&mut self) {
        // --- 1. FASE DE LEITURA (Calcula Time A) ---
        let mut saldo_a: i32 = 0;

        // Verifica se tem canastra real (limpa)
        // Note: conferir_real retorna bool, então o borrow morre imediatamente.
        let tem_canastra_real_a = self.conferir_real(0);

        // Regra dos 3 Vermelhos
        if tem_canastra_real_a {
            saldo_a += (self.tres_vermelhos_time_a.len() as i32) * 100;
        } else {
            // Regra Comum: Se não tem canastra limpa, os 3 vermelhos valem NEGATIVO
            saldo_a -= (self.tres_vermelhos_time_a.len() as i32) * 100;
        }

        // Pega as canastras (Inicia o borrow de leitura)
        let canastras_a = self.obter_canastras(0);

        for canastra in canastras_a {
            // Verifica coringa usando iterador na canastra (sem acessar self)
            let tem_coringa = canastra.iter().any(|c| c.eh_coringa());

            if tem_coringa {
                saldo_a += 100; // Canastra Suja (geralmente vale menos, ex: 100)
            } else {
                saldo_a += 200; // Canastra Limpa (geralmente vale mais, ex: 200)
                                // Nota: Se for Canastra Real (A a A) ou de 1000, ajuste aqui.
            }

            // SOMA PONTOS DAS CARTAS INDIVIDUAIS TAMBÉM?
            // No Buraco, além do prêmio da canastra, soma-se o valor de cada carta.
            let soma_cartas: i32 = canastra.iter().map(|c| c.pontos()).sum();
            saldo_a += soma_cartas;
        }
        // FIM DO ESCOPO DE 'canastras_a'. O borrow de leitura morre aqui.

        // --- 2. FASE DE ESCRITA (Atualiza o Self) ---

        // Agora o self está livre para ser modificado!
        self.pontuacao_a += saldo_a;

        // --- Repita a lógica para o Time B ---
        // (Pode criar uma função auxiliar privada 'calcular_parcial(id_time)' para não duplicar código)
    }

    fn batida(&mut self, id_jogador: u32) {}

    pub fn tentar_comprar_lixo(
        &mut self,
        jogador_id: u32,
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
            if !validar_jogo(jogo) {
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
            let minimo_necessario = self.pontos_para_descer(jogador_id); // 30, 80 ou 100

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
        self.maos[jogador_id as usize].append(&mut lixo_inteiro);

        // B. Remove as cartas usadas da mão e coloca na mesa
        for jogo in jogos_propostos {
            // Remove da mão
            for carta_jogo in &jogo {
                // Procura a carta na mão e remove a primeira ocorrência
                if let Some(pos) = self.maos[jogador_id as usize]
                    .iter()
                    .position(|c| c == carta_jogo)
                {
                    self.maos[jogador_id as usize].remove(pos);
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
        self.maos[jogador_id as usize].sort();

        Ok(())
    }

    pub fn descer(
        &mut self,
        id_jogador: u32,
        jogos_propostos: Vec<Vec<Carta>>,
    ) -> Result<(), String> {
        // --- 1. VALIDAÇÃO DE TURNO ---
        if self.turno_atual != id_jogador {
            return Err("Não é a sua vez de jogar.".to_string());
        }

        if jogos_propostos.is_empty() {
            return Err("Nenhum jogo foi enviado.".to_string());
        }

        // --- 2. VALIDAÇÃO DE POSSE (O jogador tem essas cartas?) ---
        // Para garantir que ele não está usando a mesma carta da mão em dois jogos diferentes
        // ou usando cartas que não tem, vamos clonar a mão e simular a remoção.
        let mut mao_simulada = self.maos[id_jogador as usize].clone();

        for jogo in &jogos_propostos {
            for carta_necessaria in jogo {
                // Tenta encontrar a carta na mão simulada
                if let Some(pos) = mao_simulada.iter().position(|c| c == carta_necessaria) {
                    mao_simulada.remove(pos);
                } else {
                    return Err(format!(
                        "Você não possui a carta {:?} de {:?} necessária (ou está tentando usá-la duas vezes).",
                        carta_necessaria.valor, carta_necessaria.naipe
                    ));
                }
            }
        }

        // --- 3. VALIDAÇÃO DE REGRAS DOS JOGOS ---
        for (i, jogo) in jogos_propostos.iter().enumerate() {
            if !validar_jogo(jogo) {
                return Err(format!(
                    "O jogo número {} não é uma sequência ou trinca válida.",
                    i + 1
                ));
            }
        }

        // --- 4. VALIDAÇÃO DE ABERTURA (PONTUAÇÃO) ---
        let time_id = id_jogador % 2; // 0 ou 1

        // Pega a referência dos jogos do time correto
        let jogos_na_mesa = if time_id == 0 {
            &self.jogos_time_a
        } else {
            &self.jogos_time_b
        };
        let ja_abriu = !jogos_na_mesa.is_empty();

        if !ja_abriu {
            // Calcula pontos de TODOS os jogos propostos nesta jogada
            let total_pontos_jogada: i32 = jogos_propostos
                .iter()
                .map(|jogo| jogo.iter().map(|c| c.pontos()).sum::<i32>())
                .sum();

            let pontuacao_time = if time_id == 0 {
                self.pontuacao_a
            } else {
                self.pontuacao_b
            };
            let minimo = self.pontos_para_descer(id_jogador);

            if total_pontos_jogada < minimo {
                return Err(format!(
                    "Pontuação insuficiente para abrir. Necessário: {}, Seus Jogos: {}",
                    minimo, total_pontos_jogada
                ));
            }
        }

        // --- 5. EXECUÇÃO (Se chegou aqui, está tudo certo!) ---

        // A. Remove as cartas da mão REAL do jogador
        for jogo in &jogos_propostos {
            for carta in jogo {
                // Unwrap é seguro aqui porque já validamos na "simulação" (passo 2)
                let pos = self.maos[id_jogador as usize]
                    .iter()
                    .position(|c| c == carta)
                    .unwrap();
                self.maos[id_jogador as usize].remove(pos);
            }
        }

        // B. Adiciona os jogos na mesa
        if time_id == 0 {
            self.jogos_time_a.extend(jogos_propostos);
        } else {
            self.jogos_time_b.extend(jogos_propostos);
        }

        // --- 6. CHECK DE "BATIDA EM DIRETO" ---
        // Se o jogador baixou todas as cartas e ficou sem nada na mão,
        // ele pega o morto IMEDIATAMENTE e continua jogando (sem descarte).
        if self.maos[id_jogador as usize].is_empty() {
            // Função que implementaremos a seguir para pegar o morto
            self.batida(id_jogador);
        }

        Ok(())
    }

    pub fn comprar_carta(&mut self, id_jogador: usize) {
        let carta = self.baralho.comprar();
        if let Some(c) = carta {
            self.maos[id_jogador].push(c);
            self.processar_tres_vermelhos(id_jogador);
        }
    }

    pub fn descartar_lixo(&mut self, id_jogador: usize, carta_descarte: &Carta) {
        if let Some(idx) = self.maos[id_jogador]
            .iter()
            .position(|c| c == carta_descarte)
        {
            let carta = self.maos[id_jogador].remove(idx);
            self.lixo.push(carta);
        }
    }

    /// Cria um recorte seguro do estado para um jogador específico
    pub fn gerar_visao_para(&self, jogador_alvo_id: usize) -> VisaoJogador {
        todo!();
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub enum AcaoJogador {
    ComprarBaralho,
    Descer {
        jogos: Vec<Vec<Carta>>,
    },
    ComprarLixo {
        // As cartas QUE ESTÃO NA MÃO do jogador e serão usadas junto com a do topo
        cartas_para_descer: Vec<Carta>,
    },
}

pub fn validar_jogo(cartas: &[Carta]) -> bool {
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

    // Regra Buraco: Máximo 1 coringa por jogo (seja sequência ou trinca)
    if coringas_count > 1 {
        return false;
    }

    // Se só tem coringas, é inválido
    if naturais.is_empty() {
        return false;
    }

    // --- BIFURCAÇÃO: TRINCA DE ASES VS SEQUÊNCIA ---

    // Verifica se TODAS as cartas naturais são Ases
    let sao_todos_ases = naturais.iter().all(|c| c.valor == Valor::As);

    if sao_todos_ases {
        // === ROTA 1: VALIDAÇÃO DE TRINCA DE ASES (LAVADEIRA) ===
        // Regras da Lavadeira:
        // 1. Mínimo 3 cartas (já checado no início).
        // 2. Todos são Ases (já checado no if).

        return true; // É uma lavadeira válida!
    }

    // === ROTA 2: VALIDAÇÃO DE SEQUÊNCIA (O seu código original) ===

    // PASSO B (Sequência): Validar se todos têm o MESMO naipe
    let naipe_referencia = naturais[0].naipe;
    for c in &naturais {
        if c.naipe != naipe_referencia {
            return false; // Na sequência, naipes misturados são proibidos
        }
    }

    // PASSO C (Sequência): Ordenar e Verificar Lacunas
    naturais.sort_by_key(|c| c.valor_numerico_sequencia());

    let mut lacunas_para_preencher = 0;

    for i in 0..(naturais.len() - 1) {
        let valor_atual = naturais[i].valor_numerico_sequencia();
        let valor_proximo = naturais[i + 1].valor_numerico_sequencia();

        // Checagem de segurança
        if valor_atual == 0 || valor_proximo == 0 {
            return false;
        }

        // Duplicata em sequência é inválido (ex: 4-4-5)
        if valor_atual == valor_proximo {
            return false;
        }

        // Calcula o intervalo
        // Ex: 4 e 5 -> (5 - 4 - 1) = 0 lacunas
        // Ex: 4 e 6 -> (6 - 4 - 1) = 1 lacuna
        let diff = valor_proximo - valor_atual - 1;
        lacunas_para_preencher += diff;
    }

    // PASSO D: Temos coringas suficientes?
    coringas_count >= lacunas_para_preencher
}

fn tem_coringa(jogo: &Vec<Carta>) -> bool {
    jogo.iter().any(|c| c.eh_coringa())
}

fn calcula_pontos_jogos(jogos_mesa: &Vec<Vec<Carta>>) -> i32 {
    jogos_mesa
        .iter()
        .map(|jogo| jogo.iter().map(|c| c.pontos()).sum::<i32>())
        .sum()
}
