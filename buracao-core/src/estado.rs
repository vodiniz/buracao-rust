use crate::acoes::AcaoJogador;
use crate::acoes::{DetalheJogo, VisaoJogador};
use crate::baralho::{Baralho, Carta, Naipe, Valor}; // Importa do módulo vizinho
use crate::regras::{tem_coringa, validar_jogo};
use serde::{Deserialize, Serialize};
use std::collections::HashMap; // Importa as funções puras

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
    pub pontuacao_b: i32,
    pub rodada: u32,
    pub tres_vermelhos_time_a: Vec<Carta>,
    pub tres_vermelhos_time_b: Vec<Carta>,
    pub pegou_lixo_nesta_rodada: bool,
    pub partida_encerrada: bool,
    pub proximo_id_jogo: u32,
    pub baralho_acabou_nesta_rodada: bool,
    pub comprou_nesta_rodada: bool,
}

impl EstadoJogo {
    pub fn new() -> Self {
        Self {
            baralho: Baralho::new(),
            maos: vec![Vec::new(); 4],
            turno_atual: 0,
            lixo: Vec::new(),
            jogos_time_a: HashMap::new(),
            jogos_time_b: HashMap::new(),
            pontuacao_a: 0,
            pontuacao_b: 0,
            rodada: 0,
            tres_vermelhos_time_a: Vec::new(),
            tres_vermelhos_time_b: Vec::new(),
            pegou_lixo_nesta_rodada: false,
            partida_encerrada: false,
            proximo_id_jogo: 0,
            baralho_acabou_nesta_rodada: false,
            comprou_nesta_rodada: false,
        }
    }

    pub fn preparar_proxima_rodada(&mut self) {
        // 1. Alterna quem começa (baseado na rodada anterior)
        self.rodada += 1; // Incrementa contador global de rodadas

        // A Regra 2 diz que muda quem começa.
        // Se rodada 0 começou o jogador 0.
        // Rodada 1 começa o jogador 1, etc.
        self.turno_atual = self.rodada % 4;

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

        let mut maos: Vec<Vec<Carta>> = vec![Vec::new(); 4];

        // REGRA BURACO: Geralmente são 11 cartas por jogador.
        for _ in 0..15 {
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
        jogos_do_time.iter().any(|(_, jogo)| jogo.len() >= 7)
    }

    pub fn obter_canastras(&self, id_jogador: usize) -> Vec<&Vec<Carta>> {
        let jogos_do_time = if id_jogador.is_multiple_of(2) {
            &self.jogos_time_a
        } else {
            &self.jogos_time_b
        };

        // Filtra e coleta referências
        let canastras: Vec<&Vec<Carta>> = jogos_do_time
            .values()
            .filter(|jogo| jogo.len() >= 7)
            .collect();

        canastras
    }

    pub fn contar_pontos(&mut self) {
        // 1. FASE DE LEITURA
        // Agora passamos referências para os HashMaps
        let saldo_a =
            Self::calcular_pontuacao_parcial(&self.jogos_time_a, &self.tres_vermelhos_time_a);
        let saldo_b =
            Self::calcular_pontuacao_parcial(&self.jogos_time_b, &self.tres_vermelhos_time_b);

        // 2. FASE DE ESCRITA
        self.pontuacao_a += saldo_a;
        self.pontuacao_b += saldo_b;
    }

    /// Função pura auxiliar: calcula a pontuação de um time baseada em seus jogos e 3 vermelhos.
    /// Como ela não precisa de nada do 'self' além dos argumentos, pode ser uma função associada (sem &self)
    /// ou um método de leitura.
    /// Calcula a pontuação usando HashMap<u32, Vec<Carta>>
    fn calcular_pontuacao_parcial(
        jogos: &HashMap<u32, Vec<Carta>>,
        tres_vermelhos: &[Carta],
    ) -> i32 {
        // --- Passo 1: Analisar Canastras ---

        // .values() ignora as chaves (IDs) e foca apenas nos vetores de cartas
        let tem_canastra_limpa = jogos
            .values()
            .filter(|j| j.len() >= 7)
            .any(|j| !tem_coringa(j));

        // --- Passo 2: Calcular 3 Vermelhos ---
        let qtd_3 = tres_vermelhos.len() as i32;
        let pontos_3 = if tem_canastra_limpa {
            qtd_3 * 100
        } else {
            -(qtd_3 * 100)
        };

        // --- Passo 3: Calcular Pontos dos Jogos (Cartas + Bonificações) ---
        let pontos_jogos: i32 = jogos
            .values()
            .map(|jogo| {
                // Soma o valor de cada carta individualmente
                let soma_cartas: i32 = jogo.iter().map(|c| c.pontos()).sum();

                // Calcula bônus de canastra (7+ cartas)
                let bonus = if jogo.len() >= 7 {
                    if tem_coringa(jogo) {
                        100
                    } else {
                        300
                    } //
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

        // Bonificação de batida (geralmente 100 pontos)
        if id_jogador.is_multiple_of(2) {
            self.pontuacao_a += 100;
        } else {
            self.pontuacao_b += 100;
        }

        self.partida_encerrada = true;

        // Chama a contagem final dos pontos da mesa e mãos restantes
        self.contar_pontos_final();
    }

    fn pode_bater(&self, id_jogador: usize) -> Result<(), String> {
        // Regra 1: Proibido bater se comprou o lixo nesta rodada
        if self.pegou_lixo_nesta_rodada {
            return Err("Você não pode bater na mesma rodada que comprou o lixo.".to_string());
        }

        // Regra 2: Só pode bater se tiver Canastra Real (Limpa)
        if !self.conferir_real(id_jogador) {
            return Err(
                "Para bater, seu time precisa de pelo menos uma Canastra Real.".to_string(),
            );
        }

        Ok(())
    }

    // Função auxiliar para penalizar quem ficou com cartas na mão
    fn contar_pontos_final(&mut self) {
        // Soma os pontos da mesa (canastras)
        self.contar_pontos();

        // REGRA 19: Caso as cartas do monte acabem, ninguém é penalizado
        // Se alguém bateu, self.baralho não necessariamente está vazio.
        // Se o jogo acabou porque o baralho acabou (e ninguém bateu), a penalidade é ignorada.

        // Assumindo que se self.partida_encerrada == true E baralho vazio, aplica regra 19?
        // Geralmente a regra é: Se alguém bateu, paga. Se o jogo morreu no baralho, não paga.
        let deve_penalizar = self.maos.iter().any(|mao| mao.is_empty());

        if deve_penalizar {
            for (i, mao) in self.maos.iter().enumerate() {
                let pontos_penalidade: i32 = mao.iter().map(|c| c.pontos()).sum();

                if i % 2 == 0 {
                    self.pontuacao_a -= pontos_penalidade;
                } else {
                    self.pontuacao_b -= pontos_penalidade;
                }
            }
        }
    }

    pub fn tentar_comprar_lixo(
        &mut self,
        jogador_id: u32,
        novos_jogos: Vec<Vec<Carta>>, // Jogos novos que ele está criando
        ajuntes: Vec<(u32, Vec<Carta>)>, // (ID do jogo na mesa, cartas da mão que ele vai somar)
    ) -> Result<(), String> {
        // --- 1. VALIDAÇÕES BÁSICAS ---
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

        // --- 2. VERIFICAR SE O TOPO FOI USADO E SE HÁ PELO MENOS 3 CARTAS ---

        // Verificamos nos jogos novos
        let usado_em_novo = novos_jogos
            .iter()
            .find(|jogo| jogo.iter().any(|c| c == &carta_topo_lixo) && jogo.len() >= 3);

        // Verificamos nos ajuntes
        let usado_em_ajunte = ajuntes.iter().find(|(id_jogo, cartas_somadas)| {
            let mesa = if time_id == 0 {
                &self.jogos_time_a
            } else {
                &self.jogos_time_b
            };
            if let Some(jogo_mesa) = mesa.get(id_jogo) {
                // A carta do topo tem que estar nas cartas que ele está enviando
                let contem_topo = cartas_somadas.iter().any(|c| c == &carta_topo_lixo);
                // O total (mesa + novas) deve ser >= 3 (o que sempre será verdade no Buraco, mas checamos)
                contem_topo && (jogo_mesa.len() + cartas_somadas.len() >= 3)
            } else {
                false
            }
        });

        if usado_em_novo.is_none() && usado_em_ajunte.is_none() {
            return Err(
                "Você deve usar a carta do topo em um jogo de pelo menos 3 cartas.".to_string(),
            );
        }

        // --- 3. VALIDAR INTEGRIDADE DOS JOGOS (Simulação) ---

        // Validar novos jogos
        for jogo in &novos_jogos {
            if !validar_jogo(jogo) {
                return Err("Um dos novos jogos é inválido.".to_string());
            }
        }

        // Validar ajuntes
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

        // --- 4. VALIDAÇÃO DE PONTOS DE ABERTURA (Se for o caso) ---
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
            // Nota: No Buraco, geralmente você só abre com jogos NOVOS ou ajuntes na mesma jogada
            for (_, cartas) in &ajuntes {
                total_pontos += cartas.iter().map(|c| c.pontos()).sum::<i32>();
            }

            if total_pontos < self.pontos_para_descer(jogador_id) {
                return Err("Pontos insuficientes para abrir.".to_string());
            }
        }

        // --- 5. EXECUÇÃO (Ponto de não retorno) ---

        // A. O lixo vai para a mão
        let mut cartas_lixo = self.lixo.drain(..).collect::<Vec<Carta>>();
        self.maos[jogador_idx].append(&mut cartas_lixo);

        // B. Processar Novos Jogos
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

        // C. Processar Ajuntes
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
                j.sort_by_key(|c| c.valor_numerico_sequencia()); // Opcional: manter ordenado
            }
        }
        self.comprou_nesta_rodada = true;
        self.pegou_lixo_nesta_rodada = true;
        self.maos[jogador_idx].sort();

        Ok(())
    }
    pub fn descer(
        &mut self,
        id_jogador: u32,
        jogos_propostos: Vec<Vec<Carta>>,
    ) -> Result<(), String> {
        // --- 1. VALIDAÇÕES BÁSICAS ---
        if self.turno_atual != id_jogador {
            return Err("Não é a sua vez de jogar.".to_string());
        }

        if jogos_propostos.is_empty() {
            return Err("Nenhum jogo foi enviado.".to_string());
        }

        let jogador_idx = id_jogador as usize;

        // --- 2. SIMULAÇÃO E VALIDAÇÃO DE REGRAS ---
        // Clonamos a mão para testar a jogada sem alterar o estado original
        let mut mao_simulada = self.maos[jogador_idx].clone();

        for jogo in &jogos_propostos {
            // A. Valida se o jogo em si é válido (sequência/trinca)
            if !validar_jogo(jogo) {
                return Err("Um dos jogos enviados é inválido.".to_string());
            }

            // B. Tenta remover as cartas da mão simulada
            for carta in jogo {
                if let Some(pos) = mao_simulada.iter().position(|c| c == carta) {
                    mao_simulada.remove(pos);
                } else {
                    return Err(format!("Você não possui a carta {:?}.", carta));
                }
            }
        }

        // --- 3. VALIDAÇÃO DE PONTUAÇÃO DE ABERTURA ---
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

        // --- 4. VALIDAÇÃO ANTECIPADA DA BATIDA ---
        let vai_bater = mao_simulada.is_empty();
        if vai_bater {
            // Verificamos se ele pode bater (considerando os jogos que ele está baixando AGORA)
            self.pode_bater_com_contexto(jogador_idx, &jogos_propostos)?;
        }

        // --- 5. EXECUÇÃO (Ponto de não retorno) ---

        // A. Remove as cartas da mão real
        // Como validamos na simulação, o unwrap() aqui é seguro.
        let mao_real = &mut self.maos[jogador_idx];
        for jogo in &jogos_propostos {
            for carta in jogo {
                let pos = mao_real.iter().position(|c| c == carta).unwrap();
                mao_real.remove(pos);
            }
        }

        // B. Adiciona os jogos na mesa (Usando o HashMap e IDs únicos)
        let mesa = if time_id == 0 {
            &mut self.jogos_time_a
        } else {
            &mut self.jogos_time_b
        };

        for jogo in jogos_propostos {
            let novo_id = self.proximo_id_jogo;
            self.proximo_id_jogo += 1; // Incrementa o contador global

            mesa.insert(novo_id, jogo);
        }

        // C. Finaliza a batida se necessário
        if vai_bater {
            self.batida(id_jogador);
        }

        // Opcional: manter a mão organizada
        self.maos[jogador_idx].sort();

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

        //Verifica se já tem real na mesa OU se alguma das novas é real
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

        // 1. Localizar o jogo (no time correto)
        let jogo_original = if time_id == 0 {
            self.jogos_time_a.get_mut(&id_jogo)
        } else {
            self.jogos_time_b.get_mut(&id_jogo)
        }
        .ok_or("Jogo não encontrado ou não pertence ao seu time.")?;

        // 2. Validar se o jogador tem as cartas novas na mão
        let mut mao_simulada = self.maos[jogador_idx].clone();
        for c in &cartas_novas {
            if let Some(pos) = mao_simulada.iter().position(|x| x == c) {
                mao_simulada.remove(pos);
            } else {
                return Err(format!("Você não tem a carta {:?} na mão.", c));
            }
        }

        // 3. Simular a fusão: Jogo atual + Cartas novas
        let mut jogo_simulado = jogo_original.clone();
        jogo_simulado.extend(cartas_novas.clone());

        // 4. Validar a nova formação (Aqui entra a lógica do coringa e do 4 ao A)
        if !validar_jogo(&jogo_simulado) {
            return Err("A nova formação do jogo é inválida.".to_string());
        }

        // 5. Se passou na validação, aplicar as mudanças
        // Remover da mão real
        for c in &cartas_novas {
            let pos = self.maos[jogador_idx].iter().position(|x| x == c).unwrap();
            self.maos[jogador_idx].remove(pos);
        }

        // Atualizar o jogo na mesa
        *jogo_original = jogo_simulado;

        // 6. Verificar se bateu (se a mão ficou vazia)
        if self.maos[jogador_idx].is_empty() {
            self.pode_bater_com_contexto(jogador_idx, &[])?; // [] porque não estamos criando jogos novos
            self.batida(id_jogador);
        }

        Ok(())
    }

    pub fn comprar_carta(&mut self, id_jogador: usize) -> Result<(Carta), String> {
        if self.comprou_nesta_rodada {
            return Err("Você já comprou uma carta neste turno. Jogue ou descarte.".to_string());
        }

        if self.pegou_lixo_nesta_rodada {
            return Err(
                "Você já pegou lixo nessa rodada, portanto não pode comprar uma carta".to_string(),
            );
        }

        // Verifica se ainda tem cartas no monte
        if self.baralho.restantes() > 0 {
            if let Some(c) = self.baralho.comprar() {
                self.maos[id_jogador].push(c.clone());
                self.processar_tres_vermelhos(id_jogador);
                self.comprou_nesta_rodada = true;
                // Define status para permitir jogar/descartar
                // (Assumindo que você criou o enum StatusTurno sugerido antes, senão ignore esta linha)
                // self.status_turno = StatusTurno::PodeJogarOuDescartar;
                return Ok(c);
            }
        }

        // --- REGRA 20: Baralho acabou ---

        // Verifica condições do lixo
        let lixo_vazio = self.lixo.is_empty();

        // Verifica se está travado (Regra 23)
        let lixo_travado = if let Some(topo) = self.lixo.last() {
            topo.trava_o_lixo()
        } else {
            false
        };

        if lixo_vazio || lixo_travado {
            // Se não tem baralho e o lixo não pode ser pego, ACABOU.
            self.encerrar_partida_por_esgotamento();
            return Err(
                "O baralho acabou e o lixo está vazio ou travado. Fim de jogo.".to_string(),
            );
        } else {
            // O baralho acabou, mas o lixo está disponível.
            // Marcamos a flag para saber que o jogo deve acabar logo após esse turno.
            self.baralho_acabou_nesta_rodada = true;

            return Err("O baralho acabou! Esta é a última chance. Você deve tentar comprar o lixo (fazer jogo/ajunte) ou o jogo encerrará.".to_string());
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

        // Não pode bater pegando o lixo
        // (Se ele pegou o lixo e vai ficar com 0 cartas após descarte, bloqueia)
        // Nota: A função 'tentar_comprar_lixo' seta 'pegou_lixo_nesta_rodada = true'
        if self.pegou_lixo_nesta_rodada && self.maos[jogador_idx].len() == 1 {
            // Verifica se a carta a ser descartada é a única que resta
            if self.maos[jogador_idx].contains(&carta_descarte) {
                return Err("Regra 24: Você pegou o lixo, então não pode bater (ficar sem cartas) neste turno.".to_string());
            }
        }

        // --- Lógica de Descarte Padrão ---
        let pos = self.maos[jogador_idx]
            .iter()
            .position(|c| c == &carta_descarte)
            .ok_or("Carta não encontrada.")?;

        let carta = self.maos[jogador_idx].remove(pos);
        self.lixo.push(carta);

        // Verifica batida (Regra 16 - Precisa de Real)
        if self.maos[jogador_idx].is_empty() {
            if self.conferir_real(jogador_idx) {
                self.batida(id_jogador);
                return Ok(()); // Jogo acabou por batida
            } else {
                // Rollback visual (em memória real teria que devolver a carta,
                // mas aqui retornamos erro fatal ou tratamos)
                return Err("Você ficou sem cartas mas não tem Canastra Real!".to_string());
            }
        }

        // --- AQUI ENTRA A REGRA 20 (Fim da Rodada Extra) ---
        if self.baralho_acabou_nesta_rodada {
            self.encerrar_partida_por_esgotamento();
            return Ok(());
        }

        // Passa o turno normal
        self.turno_atual = (self.turno_atual + 1) % 4;
        self.pegou_lixo_nesta_rodada = false;
        self.comprou_nesta_rodada = false;
        Ok(())
    }

    fn encerrar_partida_por_esgotamento(&mut self) {
        println!("Fim de jogo por esgotamento do baralho!");
        self.partida_encerrada = true;

        // Calcula apenas os pontos da mesa (positivos)
        // REGRA 19: Ninguém é penalizado pelas cartas na mão

        // Time A
        let saldo_a =
            Self::calcular_pontuacao_parcial(&self.jogos_time_a, &self.tres_vermelhos_time_a);
        self.pontuacao_a += saldo_a;

        // Time B
        let saldo_b =
            Self::calcular_pontuacao_parcial(&self.jogos_time_b, &self.tres_vermelhos_time_b);
        self.pontuacao_b += saldo_b;
    }

    /// Função principal que recebe a intenção do jogador e executa no Core.
    pub fn realizar_acao(&mut self, id_jogador: u32, acao: AcaoJogador) -> Result<String, String> {
        // 1. Validação de Turno e Fim de Jogo
        if self.partida_encerrada {
            return Err("A partida já encerrou.".to_string());
        }
        if self.turno_atual != id_jogador {
            return Err(format!(
                "Não é seu turno. Vez do jogador {}.",
                self.turno_atual
            ));
        }

        // 2. Roteamento da Ação
        match acao {
            AcaoJogador::ComprarBaralho => {
                // Regra: Só pode comprar se ainda não pegou carta (nem do baralho nem do lixo)
                // Vamos assumir que você tem uma flag 'comprou_nesta_rodada' ou verifica o tamanho da mão
                // Mas geralmente, a validação é feita pelo estado da máquina (Ex: Fase::Compra)

                // Chama sua função que já trata a Regra 20 (Baralho Vazio)
                let carta = self.comprar_carta(id_jogador as usize)?;
                Ok(format!("Você comprou do baralho: {}", carta))
            }

            AcaoJogador::ComprarLixo {
                novos_jogos,
                cartas_em_jogos_existentes,
            } => {
                // Aqui chamamos a lógica complexa de validação do lixo
                // A função tentar_comprar_lixo deve:
                // 1. Verificar se o lixo não está vazio.
                // 2. Verificar se a carta do topo encaixa nos 'novos_jogos' ou 'cartas_em_jogos_existentes'.
                // 3. Se sucesso, move o lixo todo para a mão e baixa os jogos indicados.

                self.tentar_comprar_lixo(id_jogador, novos_jogos, cartas_em_jogos_existentes)?;
                Ok("Lixo comprado com sucesso e jogos baixados.".to_string())
            }

            AcaoJogador::BaixarJogos { jogos } => {
                // Valida se o jogador já comprou carta (fase correta)
                // Loop para baixar múltiplos jogos
                self.descer(id_jogador, jogos)?;
                Ok("Jogos baixados com sucesso.".to_string())
            }

            AcaoJogador::Ajuntar {
                indice_jogo,
                cartas,
            } => {
                // Chama a função de inserir (que verifica se a carta cabe na sequência)
                self.ajuntar(id_jogador, indice_jogo, cartas)?;
                Ok("Cartas inseridas no jogo com sucesso.".to_string())
            }

            AcaoJogador::Descartar { carta } => {
                // Esta é a ação que finaliza o turno e checa batida/fim de baralho
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
        // 1. Anonimizar as mãos dos outros
        let qtd_cartas_jogadores: Vec<usize> = self.maos.iter().map(|mao| mao.len()).collect();

        // 2. Preparar a mão do jogador (se o ID for válido)
        let minha_mao = if (id_observador as usize) < self.maos.len() {
            self.maos[id_observador as usize].clone()
        } else {
            Vec::new() // Espectador ou erro
        };

        // 3. Converter os HashMaps de jogos para Vetores (mais fácil pro JSON)
        let converter_mesa =
            |jogos: &std::collections::HashMap<u32, Vec<Carta>>| -> Vec<DetalheJogo> {
                let mut lista: Vec<DetalheJogo> = jogos
                    .iter()
                    .map(|(id, cartas)| DetalheJogo {
                        id: *id,
                        cartas: cartas.clone(),
                        tipo: if cartas.len() >= 7 {
                            "Canastra".to_string()
                        } else {
                            "Normal".to_string()
                        },
                    })
                    .collect();
                // Ordenar por ID para a UI não ficar pulando
                lista.sort_by_key(|j| j.id);
                lista
            };

        VisaoJogador {
            meu_id: id_observador,
            minha_mao,
            posso_jogar: self.turno_atual == id_observador,

            mesa_time_a: converter_mesa(&self.jogos_time_a),
            mesa_time_b: converter_mesa(&self.jogos_time_b),

            lixo: self.lixo.last().cloned(), // Envia todo o lixo (Buraco Aberto). Se for Fechado, mude aqui.
            //
            qtd_cartas_jogadores,

            pontuacao_a: self.pontuacao_a,
            pontuacao_b: self.pontuacao_b,
            turno_atual: self.turno_atual,
            rodada: self.rodada,

            cartas_no_monte: self.baralho.restantes(),
            // Assumindo que você tem lógica de morto, senão hardcode false
            tem_morto_a: false, // Implementar logica de morto se tiver
            tem_morto_b: false,
        }
    }
}
