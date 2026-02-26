use super::state::{CartaIdentificada, GameState}; // <--- Adicionado CartaIdentificada
use crate::components::notification::{Toast, ToastType};
use buracao_core::acoes::AcaoJogador;
use buracao_core::baralho::Carta;
use futures::channel::mpsc::UnboundedSender;
use leptos::prelude::*;

#[derive(Clone)]
pub struct GameActions {
    state: GameState,
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

    fn notificar_erro_local(&self, msg: &str) {
        self.state.trigger_shake.update(|n| *n += 1);

        let id = self.state.next_toast_id.get_untracked();
        self.state.next_toast_id.set(id + 1);
        self.state.toasts.update(|t| {
            t.push(Toast {
                id,
                message: msg.to_string(),
                toast_type: ToastType::Error,
            })
        });

        let toasts = self.state.toasts;
        set_timeout(
            move || {
                toasts.update(|t| t.retain(|toast| toast.id != id));
            },
            std::time::Duration::from_secs(4),
        );
    }

    fn get_cartas_selecionadas(&self) -> Vec<Carta> {
        let ids = self.state.selected_ids.get();
        self.state.minha_mao.with(|mao| {
            mao.iter()
                .filter(|wrapper| ids.contains(&wrapper.id))
                .map(|wrapper| wrapper.carta.clone())
                .collect()
        })
    }

    // --- AÇÕES DO JOGADOR ---

    pub fn comprar_monte(&self) {
        self.enviar(AcaoJogador::ComprarBaralho);
    }

    pub fn descartar(&self) {
        let ids = self.state.selected_ids.get();

        if ids.len() != 1 {
            self.notificar_erro_local("Selecione exatamente 1 carta para descartar!");
            return;
        }

        // Pega a carta correspondente ao ID selecionado
        let carta_opt = self.state.minha_mao.with(|mao| {
            let id_alvo = *ids.iter().next().unwrap();
            mao.iter()
                .find(|c| c.id == id_alvo)
                .map(|c| c.carta.clone())
        });

        if let Some(carta) = carta_opt {
            self.enviar(AcaoJogador::Descartar { carta });
            self.state.selected_ids.update(|s| s.clear());
        }
    }

    pub fn separar(&self) {
        let mao_atual = self.state.minha_mao.get();
        let ids = self.state.selected_ids.get();

        if ids.len() < 3 {
            self.notificar_erro_local("Selecione pelo menos 3 cartas para baixar um jogo.");
            return;
        }

        // Particiona baseando-se no ID
        let (sel, resto): (Vec<_>, Vec<_>) = mao_atual
            .into_iter()
            .partition(|wrapper| ids.contains(&wrapper.id));

        let cartas_para_baixar: Vec<Carta> = sel.into_iter().map(|w| w.carta).collect();
        let nova_mao = resto; // Já é Vec<CartaIdentificada>

        self.state
            .jogos_preparados
            .update(|jogos| jogos.push(cartas_para_baixar));
        self.state.minha_mao.set(nova_mao);
        self.state.selected_ids.update(|s| s.clear());
    }

    pub fn devolver(&self, idx_jogo_preparado: usize) {
        let mut jogo_removido = None;
        self.state.jogos_preparados.update(|jogos| {
            if idx_jogo_preparado < jogos.len() {
                jogo_removido = Some(jogos.remove(idx_jogo_preparado));
            }
        });

        if let Some(cartas) = jogo_removido {
            self.state.minha_mao.update(|mao| {
                // Ao devolver, precisamos gerar IDs novos, pois essas cartas
                // tecnicamente "perderam" sua identidade visual ao ir para a mesa.
                // Usamos o contador global para isso.
                let mut next_id = self.state.unique_card_counter.get_untracked();

                for carta in cartas {
                    next_id += 1;
                    mao.push(CartaIdentificada { id: next_id, carta });
                }
                self.state.unique_card_counter.set(next_id);

                // Ordenação opcional baseada no valor da carta
                mao.sort_by(|a, b| a.carta.cmp(&b.carta));
            });
        }
    }

    pub fn organizar_mao(&self) {
        self.state.minha_mao.update(|mao| {
            mao.sort_by(|a, b| a.carta.cmp(&b.carta));
        });
    }

    pub fn confirmar_baixa(&self) {
        let jogos = self.state.jogos_preparados.get();
        if jogos.is_empty() {
            return;
        }
        self.enviar(AcaoJogador::BaixarJogos { jogos });
    }

    // --- LÓGICA DE LIXO E AJUNTES ---

    pub fn toggle_lixo_selecao(&self) {
        self.state.lixo_selecionado.update(|v| *v = !*v);
    }

    pub fn cancelar_lixo(&self) {
        self.state.lixo_selecionado.set(false);
        self.state.ajuntes_lixo_preparados.set(Vec::new());
        self.state.selected_ids.update(|s| s.clear());
    }

    pub fn confirmar_compra_lixo(&self) {
        if !self.state.lixo_selecionado.get() {
            return;
        }

        let ajuntes_guardados = self.state.ajuntes_lixo_preparados.get();
        let mut novos_jogos = self.state.jogos_preparados.get();

        let cartas_soltas = self.get_cartas_selecionadas();
        if !cartas_soltas.is_empty() {
            novos_jogos.push(cartas_soltas);
        }

        if ajuntes_guardados.is_empty() && novos_jogos.is_empty() {
            self.notificar_erro_local(
                "Para pegar o lixo, faça um jogo novo ou ajunte em um existente.",
            );
            return;
        }

        self.enviar(AcaoJogador::ComprarLixo {
            novos_jogos,
            cartas_em_jogos_existentes: ajuntes_guardados,
        });

        self.state.lixo_selecionado.set(false);
        self.state.ajuntes_lixo_preparados.set(Vec::new());
        self.state.selected_ids.update(|s| s.clear());
    }

    pub fn ajuntar(&self, idx_jogo_mesa: usize) {
        let cartas_selecionadas = self.get_cartas_selecionadas();

        if cartas_selecionadas.is_empty() {
            self.notificar_erro_local("Selecione cartas da mão primeiro para ajuntar!");
            return;
        }

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
                let ajunte_do_lixo = vec![(id_real, cartas_selecionadas)];
                let jogos_novos_guardados = self.state.jogos_preparados.get();

                self.enviar(AcaoJogador::ComprarLixo {
                    novos_jogos: jogos_novos_guardados,
                    cartas_em_jogos_existentes: ajunte_do_lixo,
                });
                self.state.lixo_selecionado.set(false);
            } else {
                self.enviar(AcaoJogador::Ajuntar {
                    indice_jogo: id_real,
                    cartas: cartas_selecionadas,
                });
            }
            self.state.selected_ids.update(|s| s.clear());
        }
    }
}
