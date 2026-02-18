use futures::channel::mpsc::UnboundedSender;
use leptos::html::Audio;
use leptos::prelude::*;
use std::collections::{HashMap, HashSet};

use buracao_core::acoes::DetalheJogo;
use buracao_core::baralho::Carta;

use crate::components::notification::Toast;
use crate::components::shortcut_manager::KeyBindings;

#[derive(Clone, Copy)]
pub struct GameState {
    // --- ESTADO DO JOGO (Vindo do Servidor) ---
    pub turno_atual_id: RwSignal<u32>,
    pub minha_mao: RwSignal<Vec<Carta>>,
    pub lixo_topo: RwSignal<Option<Carta>>,
    pub jogos_preparados: RwSignal<Vec<Vec<Carta>>>,
    pub ajuntes_lixo_preparados: RwSignal<Vec<(u32, Vec<Carta>)>>,
    pub mesa_a: RwSignal<Vec<DetalheJogo>>,
    pub mesa_b: RwSignal<Vec<DetalheJogo>>,
    pub pontuacao_a: RwSignal<i32>,
    pub pontuacao_b: RwSignal<i32>,
    pub historico_a: RwSignal<Vec<i32>>,
    pub historico_b: RwSignal<Vec<i32>>,
    pub tres_vermelhos_a: RwSignal<Vec<Carta>>,
    pub tres_vermelhos_b: RwSignal<Vec<Carta>>,
    pub meu_id: RwSignal<u32>,
    pub status_jogo: RwSignal<String>,
    pub sou_o_jogador_da_vez: RwSignal<bool>,
    pub qtd_monte: RwSignal<u32>,
    pub qtd_lixo: RwSignal<u32>,
    pub verso_monte: RwSignal<String>,
    pub mapa_nomes: RwSignal<HashMap<u32, String>>,
    pub qtd_cartas_jogadores: RwSignal<Vec<usize>>,

    // --- ESTADO LOCAL (Interface) ---
    pub lixo_selecionado: RwSignal<bool>,
    pub selected_indices: RwSignal<HashSet<usize>>,
    pub highlighted_games: RwSignal<HashSet<u32>>,
    pub toasts: RwSignal<Vec<Toast>>,
    pub next_toast_id: RwSignal<usize>, // Substituindo StoredValue para uniformidade
    pub in_game: RwSignal<bool>,
    pub player_name: RwSignal<String>,
    pub room_code: RwSignal<String>,
    pub ws_sender: RwSignal<Option<UnboundedSender<String>>>,

    // --- CONFIGURAÇÕES E VISUAL ---
    pub show_settings: RwSignal<bool>,
    pub current_theme: RwSignal<String>,
    pub card_scale: RwSignal<f32>,
    pub volume: RwSignal<f64>,
    pub key_bindings: RwSignal<KeyBindings>,
    pub show_history: RwSignal<bool>,

    // --- REFERÊNCIAS DE ÁUDIO ---
    pub my_turn_audio_ref: NodeRef<Audio>,
    pub draw_audio_ref: NodeRef<Audio>,
}

impl GameState {
    pub fn new() -> Self {
        // Lógica de carregar KeyBindings do LocalStorage (Trazida do App.rs para cá)
        let initial_keys = if let Some(win) = web_sys::window() {
            if let Ok(Some(storage)) = win.local_storage() {
                if let Ok(Some(json)) = storage.get_item("buraco_keys") {
                    serde_json::from_str(&json).unwrap_or_default()
                } else {
                    KeyBindings::default()
                }
            } else {
                KeyBindings::default()
            }
        } else {
            KeyBindings::default()
        };

        // Lógica de carregar Volume do LocalStorage
        let initial_volume = if let Some(win) = web_sys::window() {
            if let Ok(Some(storage)) = win.local_storage() {
                if let Ok(Some(vol_str)) = storage.get_item("buraco_volume") {
                    vol_str.parse::<f64>().unwrap_or(0.8)
                } else {
                    0.8
                }
            } else {
                0.8
            }
        } else {
            0.8
        };

        Self {
            // Jogo
            turno_atual_id: RwSignal::new(0),
            minha_mao: RwSignal::new(Vec::new()),
            lixo_topo: RwSignal::new(None),
            jogos_preparados: RwSignal::new(Vec::new()),
            ajuntes_lixo_preparados: RwSignal::new(Vec::new()),
            mesa_a: RwSignal::new(Vec::new()),
            mesa_b: RwSignal::new(Vec::new()),
            pontuacao_a: RwSignal::new(0),
            pontuacao_b: RwSignal::new(0),
            historico_a: RwSignal::new(Vec::new()),
            historico_b: RwSignal::new(Vec::new()),
            tres_vermelhos_a: RwSignal::new(Vec::new()),
            tres_vermelhos_b: RwSignal::new(Vec::new()),
            meu_id: RwSignal::new(0),
            status_jogo: RwSignal::new("Conectando...".to_string()),
            sou_o_jogador_da_vez: RwSignal::new(false),
            qtd_monte: RwSignal::new(0),
            qtd_lixo: RwSignal::new(0),
            verso_monte: RwSignal::new("back_r".to_string()),
            mapa_nomes: RwSignal::new(HashMap::new()),
            qtd_cartas_jogadores: RwSignal::new(Vec::new()),

            // Local
            lixo_selecionado: RwSignal::new(false),
            selected_indices: RwSignal::new(HashSet::new()),
            highlighted_games: RwSignal::new(HashSet::new()),
            toasts: RwSignal::new(Vec::new()),
            next_toast_id: RwSignal::new(0),
            in_game: RwSignal::new(false),
            player_name: RwSignal::new(String::new()),
            room_code: RwSignal::new(String::new()),
            ws_sender: RwSignal::new(None),

            // Configs
            show_settings: RwSignal::new(false),
            current_theme: RwSignal::new("/assets/cards/PaperCards".to_string()),
            card_scale: RwSignal::new(1.0),
            volume: RwSignal::new(initial_volume),
            key_bindings: RwSignal::new(initial_keys),
            show_history: RwSignal::new(false),

            // Refs
            my_turn_audio_ref: NodeRef::new(),
            draw_audio_ref: NodeRef::new(),
        }
    }
}
