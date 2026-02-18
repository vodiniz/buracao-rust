use super::state::GameState;
use buracao_core::acoes::AcaoJogador;
use buracao_core::baralho::Carta;
use futures::channel::mpsc::UnboundedSender;
use leptos::prelude::*;

#[derive(Clone)]
pub struct GameActions {
    state: GameState,
    // O sender pode vir do estado ou ser passado explicitamente.
    // Aqui assumimos que ele é passado na construção para facilitar testes.
    sender: Option<UnboundedSender<String>>,
}

impl GameActions {
    pub fn new(state: GameState, sender: Option<UnboundedSender<String>>) -> Self {
        Self { state, sender }
    }

    /// Helper privado para enviar mensagens ao WebSocket
    fn enviar(&self, acao: AcaoJogador) {
        if let Some(tx) = &self.sender {
            if let Ok(json) = serde_json::to_string(&acao) {
                let _ = tx.unbounded_send(json);
            }
        } else {
            leptos::logging::warn!("Tentativa de enviar ação sem conexão ativa.");
        }
    }

    // --- AÇÕES DO JOGADOR ---

    pub fn comprar_monte(&self) {
        self.enviar(AcaoJogador::ComprarBaralho);
    }

    pub fn descartar(&self) {
        let indices = self.state.selected_indices.get();
        if indices.len() != 1 {
            let _ = web_sys::window().and_then(|w| {
                w.alert_with_message("Selecione apenas 1 carta para descartar!")
                    .ok()
            });
            return;
        }

        let idx = *indices.iter().next().unwrap();
        // Clona a carta para não manter borrow da mão
        let carta_opt = self.state.minha_mao.with(|cartas| cartas.get(idx).cloned());

        if let Some(carta) = carta_opt {
            self.enviar(AcaoJogador::Descartar { carta });
            self.state.selected_indices.update(|s| s.clear());
        }
    }

    /// Move cartas da mão para a área de preparação ("Jogos a Baixar")
    pub fn separar(&self) {
        let mao_atual = self.state.minha_mao.get();
        let indices_set = self.state.selected_indices.get();

        if indices_set.len() < 3 {
            // Opcional: Avisar que precisa de 3 cartas
            return;
        }

        // Separa as cartas selecionadas das restantes
        let (sel_com_idx, resto_com_idx): (Vec<_>, Vec<_>) = mao_atual
            .into_iter()
            .enumerate()
            .partition(|(i, _)| indices_set.contains(i));

        let cartas_para_baixar: Vec<Carta> = sel_com_idx.into_iter().map(|(_, c)| c).collect();
        let nova_mao: Vec<Carta> = resto_com_idx.into_iter().map(|(_, c)| c).collect();

        // Atualiza estado local
        self.state
            .jogos_preparados
            .update(|jogos| jogos.push(cartas_para_baixar));
        self.state.minha_mao.set(nova_mao);
        self.state.selected_indices.update(|s| s.clear());
    }

    /// Devolve um jogo da área de preparação para a mão
    pub fn devolver(&self, idx_jogo_preparado: usize) {
        let mut jogo_removido = None;
        self.state.jogos_preparados.update(|jogos| {
            if idx_jogo_preparado < jogos.len() {
                jogo_removido = Some(jogos.remove(idx_jogo_preparado));
            }
        });

        if let Some(cartas) = jogo_removido {
            self.state.minha_mao.update(|mao| {
                mao.extend(cartas);
                mao.sort();
            });
        }
    }

    pub fn organizar_mao(&self) {
        self.state.minha_mao.update(|mao| mao.sort());
    }

    pub fn confirmar_baixa(&self) {
        let jogos = self.state.jogos_preparados.get();
        if jogos.is_empty() {
            return;
        }
        self.enviar(AcaoJogador::BaixarJogos { jogos });
        // Nota: Não limpamos 'jogos_preparados' aqui.
        // Esperamos o 'MsgServidor::Estado' (sucesso) ou 'Erro' para decidir.
    }

    // --- LÓGICA DE LIXO E AJUNTES ---

    pub fn toggle_lixo_selecao(&self) {
        self.state.lixo_selecionado.update(|v| *v = !*v);
    }

    pub fn cancelar_lixo(&self) {
        self.state.lixo_selecionado.set(false);
        self.state.ajuntes_lixo_preparados.set(Vec::new());
        self.state.selected_indices.update(|s| s.clear());
    }

    pub fn confirmar_compra_lixo(&self) {
        if !self.state.lixo_selecionado.get() {
            return;
        }

        let ajuntes_guardados = self.state.ajuntes_lixo_preparados.get();
        let mut novos_jogos = self.state.jogos_preparados.get();
        let indices = self.state.selected_indices.get();

        // Se tiver cartas selecionadas na mão, entende-se que elas formam um novo jogo com o lixo
        if !indices.is_empty() {
            let cartas_soltas: Vec<Carta> = self.state.minha_mao.with(|mao| {
                indices
                    .iter()
                    .filter_map(|&i| mao.get(i).cloned())
                    .collect()
            });
            novos_jogos.push(cartas_soltas);
        }

        if ajuntes_guardados.is_empty() && novos_jogos.is_empty() {
            let _ = web_sys::window().and_then(|w| {
                w.alert_with_message(
                    "Para pegar o lixo, faça um jogo novo ou ajunte em um existente.",
                )
                .ok()
            });
            return;
        }

        self.enviar(AcaoJogador::ComprarLixo {
            novos_jogos,
            cartas_em_jogos_existentes: ajuntes_guardados,
        });

        // Limpeza parcial otimista
        self.state.lixo_selecionado.set(false);
        self.state.ajuntes_lixo_preparados.set(Vec::new());
        self.state.selected_indices.update(|s| s.clear());
    }

    pub fn ajuntar(&self, idx_jogo_mesa: usize) {
        let indices = self.state.selected_indices.get();
        if indices.is_empty() {
            let _ = web_sys::window().and_then(|w| {
                w.alert_with_message("Selecione cartas da mão primeiro para ajuntar!")
                    .ok()
            });
            return;
        }

        let cartas_selecionadas: Vec<Carta> = self.state.minha_mao.with(|mao| {
            indices
                .iter()
                .filter_map(|&i| mao.get(i).cloned())
                .collect()
        });

        // Descobre se é Time A ou B para buscar na mesa correta
        let meu_id = self.state.meu_id.get();
        let sou_time_a = meu_id.is_multiple_of(2);

        let id_jogo_real = if sou_time_a {
            self.state
                .mesa_a
                .with(|m| m.get(idx_jogo_mesa).map(|j| j.id))
        } else {
            self.state
                .mesa_b
                .with(|m| m.get(idx_jogo_mesa).map(|j| j.id))
        };

        if let Some(id_real) = id_jogo_real {
            if self.state.lixo_selecionado.get() {
                // Se está tentando pegar o lixo fazendo ajunte
                let ajunte_do_lixo = vec![(id_real, cartas_selecionadas)];
                let jogos_novos_guardados = self.state.jogos_preparados.get();

                self.enviar(AcaoJogador::ComprarLixo {
                    novos_jogos: jogos_novos_guardados,
                    cartas_em_jogos_existentes: ajunte_do_lixo,
                });

                self.state.lixo_selecionado.set(false);
                self.state.selected_indices.update(|s| s.clear());
            } else {
                // Ajunte normal
                self.enviar(AcaoJogador::Ajuntar {
                    indice_jogo: id_real,
                    cartas: cartas_selecionadas,
                });
                self.state.selected_indices.update(|s| s.clear());
            }
        }
    }
}
