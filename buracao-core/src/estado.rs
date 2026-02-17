use crate::acoes::AcaoJogador;
use crate::acoes::{DetalheJogo, VisaoJogador};
use crate::baralho::{Baralho, Carta}; // Importa do módulo vizinho
use crate::regras::{tem_coringa, validar_jogo};
use crate::Verso;
use serde::{Deserialize, Serialize};
use std::collections::HashMap; // Importa as funções puras

// --- CENTRALIZAÇÃO DA PONTUAÇÃO (mínima mudança) ---
#[derive(Debug, Clone, Copy)]
enum PontuacaoRodada {
    /// Só pontuação da mesa (jogos + 3 vermelhos)
    ParcialMesa,
    /// Batida: mesa + bônus de batida (+100) + penalidade das mãos
    Batida { time_bateu: u32 }, // 0 = A, 1 = B
    /// Esgotamento do baralho: apenas mesa (Regra 19: sem penalidade de mão)
    EsgotamentoBaralho,
}

// --- ESTADO GLOBAL DO JOGO ---

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct EstadoJogo {
    pub baralho: Baralho,
    pub maos: Vec<Vec<Carta>>,
    pub turno_atual: u32, // 0 a 3
    pub lixo: Vec<Carta>,
    pub jogos_time_a: HashMap<u32, Vec<Carta>>,
    pub jogos_time_b: HashMap<u32, Vec<Carta>>,
    pub pontuacao_a: i32,
    pub historico_pontos_a: Vec<i32>,
    pub pontuacao_b: i32,
    pub historico_pontos_b: Vec<i32>,
    pub rodada: u32,
    pub numero_partida: u32,
    pub tres_vermelhos_time_a: Vec<Carta>,
    pub tres_vermelhos_time_b: Vec<Carta>,
    pub pegou_lixo_nesta_rodada: bool,
    pub partida_encerrada: bool,
    pub proximo_id_jogo: u32,
    pub baralho_acabou_nesta_rodada: bool,
    pub comprou_nesta_rodada: bool,
    pub qtd_monte: u32,
    pub qtd_lixo: u32,
    pub verso_topo: Option<Verso>,
}

impl EstadoJogo {
    pub fn new() -> Self {
        let mut baralho_inicial = Baralho::new();
        baralho_inicial.shuffle(); // Opcional: embaralhar logo no início se quiser

        let verso_inicial = baralho_inicial.cartas.last().map(|c| c.verso);

        if std::env::var("MODO_TESTE").is_ok() {
            println!("⚠️  [DEBUG] INICIANDO EM MODO DE TESTE (BATERIA) ⚠️");
            return crate::tests::gerar_jogo_teste_bateria();
        }

        let mut jogo = Self {
            // <--- Crie uma variável mutável 'jogo'
            baralho: baralho_inicial,
            maos: vec![Vec::new(); 4], // Começa vazio...
            turno_atual: 0,
            lixo: Vec::new(),
            jogos_time_a: HashMap::new(),
            jogos_time_b: HashMap::new(),
            pontuacao_a: 0,
            historico_pontos_a: Vec::new(),
            pontuacao_b: 0,
            historico_pontos_b: Vec::new(),
            rodada: 0,
            numero_partida: 0,
            tres_vermelhos_time_a: Vec::new(),
            tres_vermelhos_time_b: Vec::new(),
            pegou_lixo_nesta_rodada: false,
            partida_encerrada: false,
            proximo_id_jogo: 0,
            baralho_acabou_nesta_rodada: false,
            comprou_nesta_rodada: false,
            qtd_monte: 0,
            qtd_lixo: 0,
            verso_topo: verso_inicial,
        };

        // --- A CORREÇÃO MÁGICA ---
        // Distribuímos as cartas ANTES de devolver o jogo para o servidor
        jogo.dar_cartas();

        jogo // Retorna o jogo pronto
    }

    pub fn preparar_proxima_rodada(&mut self) {
        // 1. Alterna quem começa (baseado na rodada anterior)
        self.numero_partida += 1;
        // self.rodada = 0; // Incrementa contador global de rodadas

        // A Regra 2 diz que muda quem começa.
        // Se rodada 0 começou o jogador 0.
        // Rodada 1 começa o jogador 1, etc.

        self.turno_atual = self.numero_partida % 4;

        // 2. Limpa a mesa
        self.baralho = Baralho::new();
        self.lixo.clear();
        self.jogos_time_a.clear();
        self.jogos_time_b.clear();
        self.tres_vermelhos_time_a.clear();
        self.tres_vermelhos_time_b.clear();
        self.maos = vec![Vec::new(); 4];
        self.pegou_lixo_nesta_rodada = false;
        self.partida_encerrada = false;
        self.baralho_acabou_nesta_rodada = false;

        // 3. Distribui cartas novamente
        self.dar_cartas();
    }

    pub fn dar_cartas(&mut self) {
        self.baralho.shuffle();

        // 1. Define quem é o "mão" (quem começa a partida e recebe a primeira carta)
        // Usamos numero_partida, pois rodada zera a cada jogo.
        let id_mao = (self.numero_partida % 4) as usize;

        let mut maos: Vec<Vec<Carta>> = vec![Vec::new(); 4];

        // 2. Distribuição "Um por um" (Dealing one by one)
        // Total de cartas a distribuir: 15 cartas * 4 jogadores = 60 cartas
        for i in 0..(15 * 4) {
            let jogador_da_vez = (id_mao + i) % 4;

            if let Some(carta) = self.baralho.comprar() {
                maos[jogador_da_vez].push(carta);
            }
        }

        self.maos = maos;

        // 3. Processar 3 Vermelhos (Também seguindo a ordem do mão)
        for i in 0..4 {
            let jogador_da_vez = (id_mao + i) % 4;
            self.processar_tres_vermelhos(jogador_da_vez);
        }

        // 4. Atualiza metadados do monte
        self.qtd_monte = self.baralho.cartas.len() as u32;
        self.verso_topo = self.baralho.cartas.last().map(|c| c.verso);
    }

    pub fn processar_tres_vermelhos(&mut self, jogador_id: usize) {
        loop {
            let (novos_tres_vermelhos, resto_da_mao): (Vec<Carta>, Vec<Carta>) = self.maos
                [jogador_id]
                .drain(..)
                .partition(|c| c.eh_tres_vermelho());

            self.maos[jogador_id] = resto_da_mao;

            if novos_tres_vermelhos.is_empty() {
                break;
            }

            let qtd_reposicao = novos_tres_vermelhos.len();

            let time_id = jogador_id % 2;
            if time_id == 0 {
                self.tres_vermelhos_time_a.extend(novos_tres_vermelhos);
            } else {
                self.tres_vermelhos_time_b.extend(novos_tres_vermelhos);
            }

            for _ in 0..qtd_reposicao {
                if let Some(carta) = self.baralho.comprar() {
                    self.maos[jogador_id].push(carta);
                }
            }
        }

        // self.maos[jogador_id].sort();
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
        let jogos_do_time = if id_jogador.is_multiple_of(2) {
            &self.jogos_time_a
        } else {
            &self.jogos_time_b
        };

        jogos_do_time.iter().any(|(_, jogo)| jogo.len() >= 7)
    }

    pub fn obter_canastras(&self, id_jogador: usize) -> Vec<&Vec<Carta>> {
        let jogos_do_time = if id_jogador.is_multiple_of(2) {
            &self.jogos_time_a
        } else {
            &self.jogos_time_b
        };

        let canastras: Vec<&Vec<Carta>> = jogos_do_time
            .values()
            .filter(|jogo| jogo.len() >= 7)
            .collect();

        canastras
    }

    // ------------------------------
    // CENTRALIZAÇÃO AQUI (mínimo)
    // ------------------------------
    fn aplicar_pontuacao_rodada(&mut self, modo: PontuacaoRodada) {
        // 1) Sempre calcula a pontuação da mesa (jogos + 3 vermelhos)
        let mut saldo_a =
            Self::calcular_pontuacao_parcial(&self.jogos_time_a, &self.tres_vermelhos_time_a);
        let mut saldo_b =
            Self::calcular_pontuacao_parcial(&self.jogos_time_b, &self.tres_vermelhos_time_b);

        match modo {
            PontuacaoRodada::ParcialMesa => {
                // só mesa
            }
            PontuacaoRodada::EsgotamentoBaralho => {
                // Regra 19: sem penalidade de mão
            }
            PontuacaoRodada::Batida { time_bateu } => {
                // (3) Caso você bata, ganha 100 pontos
                if time_bateu == 0 {
                    saldo_a += 100;
                } else {
                    saldo_b += 100;
                }

                // (1) Penalidade: quando alguém bate, o time é penalizado pela pontuação na mão
                // Mantendo a estrutura/ideia original: só penaliza se existir mão vazia
                let deve_penalizar = self.maos.iter().any(|mao| mao.is_empty());

                if deve_penalizar {
                    for (i, mao) in self.maos.iter().enumerate() {
                        let pontos_penalidade: i32 = mao.iter().map(|c| c.pontos()).sum();

                        if i % 2 == 0 {
                            saldo_a -= pontos_penalidade;
                        } else {
                            saldo_b -= pontos_penalidade;
                        }
                    }
                }
            }
        }

        // 2) Aplicar e registrar no histórico (padronizado)
        self.pontuacao_a += saldo_a;
        self.historico_pontos_a.push(saldo_a);

        self.pontuacao_b += saldo_b;
        self.historico_pontos_b.push(saldo_b);
    }

    // Mantém o nome original (histórico), mas agora centraliza
    pub fn contar_pontos(&mut self) {
        self.aplicar_pontuacao_rodada(PontuacaoRodada::ParcialMesa);
    }

    /// Função pura auxiliar: calcula a pontuação de um time baseada em seus jogos e 3 vermelhos.
    /// (2) Caso seu time não tenha uma real, você é penalizado em -100 pontos para cada tres vermelho
    ///     e caso tenha você ganha 100 pontos para cada tres.
    fn calcular_pontuacao_parcial(
        jogos: &HashMap<u32, Vec<Carta>>,
        tres_vermelhos: &[Carta],
    ) -> i32 {
        let tem_canastra_limpa = jogos
            .values()
            .filter(|j| j.len() >= 7)
            .any(|j| !tem_coringa(j));

        let qtd_3 = tres_vermelhos.len() as i32;
        let pontos_3 = if tem_canastra_limpa {
            qtd_3 * 100
        } else {
            -(qtd_3 * 100)
        };

        let pontos_jogos: i32 = jogos
            .values()
            .map(|jogo| {
                let soma_cartas: i32 = jogo.iter().map(|c| c.pontos()).sum();

                let bonus = if jogo.len() >= 7 {
                    if tem_coringa(jogo) {
                        100
                    } else {
                        300
                    }
                } else {
                    0
                };

                soma_cartas + bonus
            })
            .sum();

        pontos_3 + pontos_jogos
    }

    pub fn batida(&mut self, id_jogador: u32) {
        println!("Jogador {} bateu!", id_jogador);

        self.partida_encerrada = true;

        // Centraliza: batida = mesa + bônus + penalidade
        let time_bateu = id_jogador % 2; // 0=A, 1=B
        self.contar_pontos_final(time_bateu);

        // resetar_jogo
        // self.resetar_jogo();
    }

    // Mantém o nome original (histórico), mas agora centraliza
    // (mudança mínima: recebe qual time bateu, pois o bônus +100 depende disso)
    fn contar_pontos_final(&mut self, time_bateu: u32) {
        self.aplicar_pontuacao_rodada(PontuacaoRodada::Batida { time_bateu });
    }

    pub fn tentar_comprar_lixo(
        &mut self,
        jogador_id: u32,
        mut novos_jogos: Vec<Vec<Carta>>,
        mut ajuntes: Vec<(u32, Vec<Carta>)>,
    ) -> Result<(), String> {
        if self.turno_atual != jogador_id {
            return Err("Não é seu turno".to_string());
        }

        if self.comprou_nesta_rodada {
            return Err(
                "Você já comprou uma carta nessa rodada, portanto não pode pegar lixo.".to_string(),
            );
        }

        let carta_topo_lixo = self.lixo.last().ok_or("Lixo vazio")?.clone();

        if carta_topo_lixo.trava_o_lixo() {
            return Err("O lixo está travado (3 Preto, 2 ou Joker).".to_string());
        }

        let time_id = jogador_id % 2;
        let jogador_idx = jogador_id as usize;

        let mut lixo_usado = false;

        for jogo in &mut novos_jogos {
            if !lixo_usado {
                let mut jogo_teste = jogo.clone();
                jogo_teste.push(carta_topo_lixo.clone());

                if validar_jogo(&jogo_teste) {
                    jogo.push(carta_topo_lixo.clone());
                    lixo_usado = true;
                }
            }
        }

        if !lixo_usado {
            let mesa_jogos = if time_id == 0 {
                &self.jogos_time_a
            } else {
                &self.jogos_time_b
            };

            for (id_jogo, cartas_somadas) in &mut ajuntes {
                if let Some(jogo_mesa) = mesa_jogos.get(id_jogo) {
                    let mut jogo_teste = jogo_mesa.clone();
                    jogo_teste.extend(cartas_somadas.clone());
                    jogo_teste.push(carta_topo_lixo.clone());

                    if validar_jogo(&jogo_teste) {
                        cartas_somadas.push(carta_topo_lixo.clone());
                        lixo_usado = true;
                        break;
                    }
                }
            }
        }

        if !lixo_usado {
            return Err(
                "Você deve usar a carta do topo em um jogo válido (novo ou existente).".to_string(),
            );
        }

        for jogo in &novos_jogos {
            if !validar_jogo(jogo) {
                return Err("Um dos novos jogos é inválido.".to_string());
            }
        }

        for (id_jogo, cartas_somadas) in &ajuntes {
            let mesa = if time_id == 0 {
                &self.jogos_time_a
            } else {
                &self.jogos_time_b
            };
            let jogo_mesa = mesa.get(id_jogo).ok_or("Jogo de ajunte não encontrado.")?;

            let mut jogo_combinado = jogo_mesa.clone();
            jogo_combinado.extend(cartas_somadas.clone());

            if !validar_jogo(&jogo_combinado) {
                return Err("Um dos ajuntes resultou em um jogo inválido.".to_string());
            }
        }

        let ja_abriu = if time_id == 0 {
            !self.jogos_time_a.is_empty()
        } else {
            !self.jogos_time_b.is_empty()
        };

        if !ja_abriu {
            let mut total_pontos = 0;
            for j in &novos_jogos {
                total_pontos += j.iter().map(|c| c.pontos()).sum::<i32>();
            }

            for (_, cartas) in &ajuntes {
                total_pontos += cartas.iter().map(|c| c.pontos()).sum::<i32>();
            }

            if total_pontos < self.pontos_para_descer(jogador_id) {
                return Err(format!(
                    "Pontos insuficientes para abrir. Necessário: {}, Obtido: {}",
                    self.pontos_para_descer(jogador_id),
                    total_pontos
                ));
            }
        }

        let mut cartas_lixo = self.lixo.drain(..).collect::<Vec<Carta>>();
        self.maos[jogador_idx].append(&mut cartas_lixo);
        self.qtd_lixo = self.lixo.len() as u32;

        for jogo in novos_jogos {
            for carta in &jogo {
                let pos = self.maos[jogador_idx]
                    .iter()
                    .position(|c| c == carta)
                    .unwrap();
                self.maos[jogador_idx].remove(pos);
            }
            let id = self.proximo_id_jogo;
            self.proximo_id_jogo += 1;
            let mesa = if time_id == 0 {
                &mut self.jogos_time_a
            } else {
                &mut self.jogos_time_b
            };
            mesa.insert(id, jogo);
        }

        for (id_jogo, cartas_novas) in ajuntes {
            for carta in &cartas_novas {
                let pos = self.maos[jogador_idx]
                    .iter()
                    .position(|c| c == carta)
                    .unwrap();
                self.maos[jogador_idx].remove(pos);
            }
            let mesa = if time_id == 0 {
                &mut self.jogos_time_a
            } else {
                &mut self.jogos_time_b
            };
            if let Some(j) = mesa.get_mut(&id_jogo) {
                j.extend(cartas_novas);
                j.sort_by_key(|c| c.valor_numerico_sequencia());
            }
        }

        self.comprou_nesta_rodada = true;
        self.pegou_lixo_nesta_rodada = true;

        Ok(())
    }

    pub fn descer(
        &mut self,
        id_jogador: u32,
        jogos_propostos: Vec<Vec<Carta>>,
    ) -> Result<(), String> {
        if self.turno_atual != id_jogador {
            return Err("Não é a sua vez de jogar.".to_string());
        }

        if jogos_propostos.is_empty() {
            return Err("Nenhum jogo foi enviado.".to_string());
        }

        let jogador_idx = id_jogador as usize;

        let mut mao_simulada = self.maos[jogador_idx].clone();

        for jogo in &jogos_propostos {
            if !validar_jogo(jogo) {
                return Err("Um dos jogos enviados é inválido.".to_string());
            }

            for carta in jogo {
                if let Some(pos) = mao_simulada.iter().position(|c| c == carta) {
                    mao_simulada.remove(pos);
                } else {
                    return Err(format!("Você não possui a carta {:?}.", carta));
                }
            }
        }

        let time_id = id_jogador % 2;
        let ja_abriu = if time_id == 0 {
            !self.jogos_time_a.is_empty()
        } else {
            !self.jogos_time_b.is_empty()
        };

        if !ja_abriu {
            let total_pontos: i32 = jogos_propostos
                .iter()
                .map(|jogo| jogo.iter().map(|c| c.pontos()).sum::<i32>())
                .sum();

            if total_pontos < self.pontos_para_descer(id_jogador) {
                return Err("Pontuação insuficiente para abrir o jogo.".to_string());
            }
        }

        let vai_bater = mao_simulada.is_empty();
        if vai_bater {
            self.pode_bater_com_contexto(jogador_idx, &jogos_propostos)?;
        }

        let mao_real = &mut self.maos[jogador_idx];
        for jogo in &jogos_propostos {
            for carta in jogo {
                let pos = mao_real.iter().position(|c| c == carta).unwrap();
                mao_real.remove(pos);
            }
        }

        let mesa = if time_id == 0 {
            &mut self.jogos_time_a
        } else {
            &mut self.jogos_time_b
        };

        for jogo in jogos_propostos {
            let novo_id = self.proximo_id_jogo;
            self.proximo_id_jogo += 1;

            mesa.insert(novo_id, jogo);
        }

        if vai_bater {
            self.batida(id_jogador);
        }

        Ok(())
    }

    fn pode_bater_com_contexto(
        &self,
        id_jogador: usize,
        novos_jogos: &[Vec<Carta>],
    ) -> Result<(), String> {
        if self.pegou_lixo_nesta_rodada {
            return Err("Proibido bater após comprar o lixo.".to_string());
        }

        let tem_real_mesa = self.conferir_real(id_jogador);
        let tem_real_novos = novos_jogos.iter().any(|j| j.len() >= 7 && !tem_coringa(j));

        if !tem_real_mesa && !tem_real_novos {
            return Err("Você precisa de pelo menos uma Canastra Real para bater.".to_string());
        }

        Ok(())
    }

    pub fn ajuntar(
        &mut self,
        id_jogador: u32,
        id_jogo: u32,
        cartas_novas: Vec<Carta>,
    ) -> Result<(), String> {
        let jogador_idx = id_jogador as usize;
        let time_id = id_jogador % 2;

        let jogo_original = if time_id == 0 {
            self.jogos_time_a.get_mut(&id_jogo)
        } else {
            self.jogos_time_b.get_mut(&id_jogo)
        }
        .ok_or("Jogo não encontrado ou não pertence ao seu time.")?;

        let mut mao_simulada = self.maos[jogador_idx].clone();
        for c in &cartas_novas {
            if let Some(pos) = mao_simulada.iter().position(|x| x == c) {
                mao_simulada.remove(pos);
            } else {
                return Err(format!("Você não tem a carta {:?} na mão.", c));
            }
        }

        let mut jogo_simulado = jogo_original.clone();
        jogo_simulado.extend(cartas_novas.clone());

        if !validar_jogo(&jogo_simulado) {
            return Err("A nova formação do jogo é inválida.".to_string());
        }

        for c in &cartas_novas {
            let pos = self.maos[jogador_idx].iter().position(|x| x == c).unwrap();
            self.maos[jogador_idx].remove(pos);
        }

        *jogo_original = jogo_simulado;

        if self.maos[jogador_idx].is_empty() {
            self.pode_bater_com_contexto(jogador_idx, &[])?;
            self.batida(id_jogador);
        }

        Ok(())
    }

    pub fn comprar_carta(&mut self, id_jogador: usize) -> Result<Carta, String> {
        if self.comprou_nesta_rodada {
            return Err("Você já comprou uma carta neste turno. Jogue ou descarte.".to_string());
        }

        if self.pegou_lixo_nesta_rodada {
            return Err(
                "Você já pegou lixo nessa rodada, portanto não pode comprar uma carta".to_string(),
            );
        }

        if self.baralho.restantes() > 0 {
            if let Some(c) = self.baralho.comprar() {
                self.maos[id_jogador].push(c.clone());
                self.processar_tres_vermelhos(id_jogador);
                self.comprou_nesta_rodada = true;
                self.qtd_monte = self.baralho.cartas.len() as u32;
                self.verso_topo = self.baralho.cartas.last().map(|c| c.verso);
                return Ok(c);
            }
        }

        let lixo_vazio = self.lixo.is_empty();

        let lixo_travado = if let Some(topo) = self.lixo.last() {
            topo.trava_o_lixo()
        } else {
            false
        };

        if lixo_vazio || lixo_travado {
            self.encerrar_partida_por_esgotamento();
            Err("O baralho acabou e o lixo está vazio ou travado. Fim de jogo.".to_string())
        } else {
            self.baralho_acabou_nesta_rodada = true;

            Err("O baralho acabou! Esta é a última chance. Você deve tentar comprar o lixo (fazer jogo/ajunte) ou o jogo encerrará.".to_string())
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

    pub fn descartar_e_passar_turno(
        &mut self,
        id_jogador: u32,
        carta_descarte: Carta,
    ) -> Result<(), String> {
        if self.turno_atual != id_jogador {
            return Err("Não é seu turno".to_string());
        }

        if !self.comprou_nesta_rodada && !self.pegou_lixo_nesta_rodada {
            return Err(
                "Você precisa comprar do baralho ou pegar o lixo antes de descartar.".to_string(),
            );
        }

        let jogador_idx = id_jogador as usize;

        if self.pegou_lixo_nesta_rodada && self.maos[jogador_idx].len() == 1 {
            if self.maos[jogador_idx].contains(&carta_descarte) {
                return Err("Regra 24: Você pegou o lixo, então não pode bater (ficar sem cartas) neste turno.".to_string());
            }
        }

        let pos = self.maos[jogador_idx]
            .iter()
            .position(|c| c == &carta_descarte)
            .ok_or("Carta não encontrada.")?;

        let carta = self.maos[jogador_idx].remove(pos);
        self.lixo.push(carta);

        if self.maos[jogador_idx].is_empty() {
            if self.conferir_real(jogador_idx) {
                self.batida(id_jogador);
                return Ok(());
            } else {
                return Err("Você ficou sem cartas mas não tem Canastra Real!".to_string());
            }
        }

        if self.baralho_acabou_nesta_rodada {
            self.encerrar_partida_por_esgotamento();
            return Ok(());
        }

        self.rodada += 1;
        self.turno_atual = (self.turno_atual + 1) % 4;
        self.pegou_lixo_nesta_rodada = false;
        self.comprou_nesta_rodada = false;
        self.qtd_lixo = self.lixo.len() as u32;
        Ok(())
    }

    fn encerrar_partida_por_esgotamento(&mut self) {
        println!("Fim de jogo por esgotamento do baralho!");
        self.partida_encerrada = true;

        // Centraliza: esgotamento = apenas mesa (e agora registra histórico também)
        self.aplicar_pontuacao_rodada(PontuacaoRodada::EsgotamentoBaralho);
    }

    /// Função principal que recebe a intenção do jogador e executa no Core.
    pub fn realizar_acao(&mut self, id_jogador: u32, acao: AcaoJogador) -> Result<String, String> {
        if self.partida_encerrada {
            return Err("A partida já encerrou.".to_string());
        }
        if self.turno_atual != id_jogador {
            return Err(format!(
                "Não é seu turno. Vez do jogador {}.",
                self.turno_atual
            ));
        }

        match acao {
            AcaoJogador::ComprarBaralho => {
                let carta = self.comprar_carta(id_jogador as usize)?;
                Ok(format!("Você comprou do baralho: {}", carta))
            }

            AcaoJogador::ComprarLixo {
                novos_jogos,
                cartas_em_jogos_existentes,
            } => {
                self.tentar_comprar_lixo(id_jogador, novos_jogos, cartas_em_jogos_existentes)?;
                Ok("Lixo comprado com sucesso e jogos baixados.".to_string())
            }

            AcaoJogador::BaixarJogos { jogos } => {
                self.descer(id_jogador, jogos)?;
                Ok("Jogos baixados com sucesso.".to_string())
            }

            AcaoJogador::Ajuntar {
                indice_jogo,
                cartas,
            } => {
                self.ajuntar(id_jogador, indice_jogo, cartas)?;
                Ok("Cartas inseridas no jogo com sucesso.".to_string())
            }

            AcaoJogador::Descartar { carta } => {
                self.descartar_e_passar_turno(id_jogador, carta)?;

                if self.partida_encerrada {
                    Ok("Fim de jogo!".to_string())
                } else {
                    Ok("Carta descartada. Turno passou.".to_string())
                }
            }

            AcaoJogador::Mensagem { texto } => {
                println!("{}", texto);
                Ok("Mensagem enviada".to_string())
            }
        }
    }

    pub fn gerar_visao_para(&self, id_observador: u32) -> VisaoJogador {
        let qtd_cartas_jogadores: Vec<usize> = self.maos.iter().map(|mao| mao.len()).collect();

        let minha_mao = if (id_observador as usize) < self.maos.len() {
            self.maos[id_observador as usize].clone()
        } else {
            Vec::new()
        };

        let converter_mesa =
            |jogos: &std::collections::HashMap<u32, Vec<Carta>>| -> Vec<DetalheJogo> {
                let mut lista: Vec<DetalheJogo> = jogos
                    .iter()
                    .map(|(id, cartas)| DetalheJogo {
                        id: *id,
                        cartas: cartas.clone(),
                    })
                    .collect();
                lista.sort_by_key(|j| j.id);
                lista
            };

        VisaoJogador {
            meu_id: id_observador,
            minha_mao,
            posso_jogar: self.turno_atual == id_observador,

            mesa_time_a: converter_mesa(&self.jogos_time_a),
            mesa_time_b: converter_mesa(&self.jogos_time_b),
            tres_vermelho_time_a: self.tres_vermelhos_time_a.clone(),
            tres_vermelho_time_b: self.tres_vermelhos_time_b.clone(),

            lixo: self.lixo.last().cloned(),
            qtd_cartas_jogadores,

            pontuacao_a: self.pontuacao_a,
            historico_pontos_a: self.historico_pontos_a.clone(),
            pontuacao_b: self.pontuacao_b,
            historico_pontos_b: self.historico_pontos_b.clone(),
            turno_atual: self.turno_atual,
            rodada: self.rodada,

            cartas_no_monte: self.baralho.restantes(),

            qtd_lixo: self.qtd_lixo,
            qtd_monte: self.qtd_monte,
            verso_topo: self.verso_topo,
        }
    }

    pub fn gerar_visao_para_jogador(&self, id_observador: u32) -> VisaoJogador {
        let mesa_a = self.converter_mesa_para_detalhe(&self.jogos_time_a);
        let mesa_b = self.converter_mesa_para_detalhe(&self.jogos_time_b);

        let qtd_cartas: Vec<usize> = self.maos.iter().map(|mao| mao.len()).collect();

        VisaoJogador {
            meu_id: id_observador,
            minha_mao: self.maos[id_observador as usize].clone(),
            posso_jogar: self.turno_atual == id_observador,

            mesa_time_a: mesa_a,
            mesa_time_b: mesa_b,

            tres_vermelho_time_a: self.tres_vermelhos_time_a.clone(),
            tres_vermelho_time_b: self.tres_vermelhos_time_b.clone(),

            lixo: self.lixo.last().cloned(),

            qtd_cartas_jogadores: qtd_cartas,

            pontuacao_a: self.pontuacao_a,
            historico_pontos_a: self.historico_pontos_a.clone(),
            pontuacao_b: self.pontuacao_b,
            historico_pontos_b: self.historico_pontos_b.clone(),
            turno_atual: self.turno_atual,
            rodada: self.rodada,
            cartas_no_monte: self.baralho.restantes(),

            qtd_lixo: self.qtd_lixo,
            qtd_monte: self.qtd_monte,
            verso_topo: self.verso_topo,
        }
    }

    fn converter_mesa_para_detalhe(
        &self,
        mesa: &std::collections::HashMap<u32, Vec<Carta>>,
    ) -> Vec<DetalheJogo> {
        mesa.iter()
            .map(|(id, cartas)| DetalheJogo {
                id: *id,
                cartas: cartas.clone(),
            })
            .collect()
    }

    pub fn resetar_jogo(&mut self) {
        println!("🔄 Resetando jogo (Modo Padrão)...");

        self.jogos_time_a.clear();
        self.jogos_time_b.clear();

        self.tres_vermelhos_time_a.clear();
        self.tres_vermelhos_time_b.clear();

        self.lixo.clear();
        self.maos.clear();

        self.baralho = Baralho::new();
        self.turno_atual = self.numero_partida % 4;

        self.dar_cartas();

        self.rodada = 0;
        self.comprou_nesta_rodada = false;
        self.pegou_lixo_nesta_rodada = false;

        self.partida_encerrada = false;
        self.proximo_id_jogo += 1;
        self.baralho_acabou_nesta_rodada = false;
        self.comprou_nesta_rodada = false;
        self.qtd_monte = self.baralho.cartas.len() as u32;
        self.qtd_lixo = 0;
        self.verso_topo = self.baralho.cartas.last().map(|c| c.verso);
    }
}
