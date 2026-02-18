use leptos::prelude::window;
use leptos::prelude::*;

// Módulos do Jogo (Refatorados)
use crate::game::actions::GameActions;
use crate::game::network::setup_websocket;
use crate::game::state::GameState;

// Componentes
use crate::components::board::Board;
use crate::components::controls::GameControls;
use crate::components::hand::Hand;
use crate::components::login::LoginScreen;
use crate::components::notification::NotificationToast;
use crate::components::scoreboard::{ScoreHistoryModal, Scoreboard};
use crate::components::settings::SettingsModal;
use crate::components::shortcut_manager::ShortcutManager;
use crate::components::table::Table;
use crate::components::turn_indicator::TurnIndicator;

// Utils
use crate::utils::assets::get_card_path;
use crate::utils::mappers::carta_para_asset;

// Core
use buracao_core::baralho::Carta;

// --- COMPONENTES AUXILIARES LOCAIS ---

#[component]
fn CardImage(
    carta: Carta,
    #[prop(default = "50px")] width: &'static str,
    theme: String,
) -> impl IntoView {
    let id = carta_para_asset(&carta);
    let src = get_card_path(&id, &theme);
    view! { <img src=src style=format!("width: {}; height: auto;", width) /> }
}

// --- HELPER DE DEVICE ID ---
fn get_or_create_device_id() -> String {
    let window = web_sys::window().expect("no global `window` exists");
    let storage = window
        .local_storage()
        .ok()
        .flatten()
        .expect("no local storage");

    if let Ok(Some(id)) = storage.get_item("buraco_device_id") {
        id
    } else {
        let new_id = format!("user_{}", rand::random::<u32>());
        let _ = storage.set_item("buraco_device_id", &new_id);
        new_id
    }
}

// --- COMPONENTE PRINCIPAL ---

#[component]
pub fn App() -> impl IntoView {
    // 1. INICIALIZAÇÃO DO ESTADO GLOBAL
    let state = GameState::new();
    let device_id = get_or_create_device_id();

    // 2. CONFIGURAÇÃO DE REDE (WEBSOCKET)
    // Monitora 'in_game'. Se virar true, conecta.
    Effect::new(move |_| {
        if state.in_game.get() {
            let tx = setup_websocket(state, device_id.clone());
            state.ws_sender.set(tx);
        }
    });

    // 3. HELPER DE AÇÕES
    // Cria uma instância de GameActions com o sender atual.
    // Usamos um closure para sempre pegar o sender mais recente.
    let actions = move || GameActions::new(state, state.ws_sender.get());

    // 4. CALLBACKS DA UI

    let ao_entrar = Callback::new(move |(nome, sala)| {
        state.player_name.set(nome);
        state.room_code.set(sala);
        state.in_game.set(true);
    });

    let acao_sair = move |_| {
        state.in_game.set(false);
        let _ = window().location().reload();
    };

    let acao_resetar = move |_| {
        let window = window();
        if let Ok(Some(storage)) = window.local_storage() {
            let _ = storage.remove_item("buraco_device_id");
        }
        let _ = window.location().reload();
    };

    // Cálculos de estilo
    let hand_card_width =
        Signal::derive(move || format!("{}px", (100.0 * state.card_scale.get()) as i32));
    let board_width =
        Signal::derive(move || format!("{}px", (90.0 * state.card_scale.get()) as i32));
    let table_width =
        Signal::derive(move || format!("{}px", (80.0 * state.card_scale.get()) as i32));

    let e_minha_vez = move || state.sou_o_jogador_da_vez.get();

    // --- RENDERIZAÇÃO ---
    view! {
        <Show when=move || state.in_game.get() fallback=move || view! { <LoginScreen on_enter=ao_entrar /> }>
            <div style=move || {
                let bg = if e_minha_vez() { "#388e3c" } else { "#1b5e20" };
                format!("background-color: {}; height: 100vh; display: flex; flex-direction: column; font-family: sans-serif; color: white; overflow: hidden; transition: background-color 0.5s;", bg)
            }>

                // === HEADER ===
                <div style="flex-shrink: 0; background: rgba(0,0,0,0.2); padding: 10px 20px; display: flex; justify-content: space-between; align-items: center; box-shadow: 0 2px 4px rgba(0,0,0,0.2);">
                    <div style="display: flex; flex-direction: column; align-items: flex-start;">
                        <h1 style="margin: 0; font-size: 1.5rem; line-height: 1.2;">"Buracão Web"</h1>
                        <div style="display: flex; align-items: center; gap: 10px;">
                            <small style="opacity: 0.8; font-size: 0.85rem;">
                                {move || {
                                    let id = state.meu_id.get();
                                    let time = if id.is_multiple_of(2) { "Time A" } else { "Time B" };
                                    format!("Meu ID: {} ({})", id, time)
                                }}
                            </small>
                            <button
                                on:click=move |_| state.show_settings.set(true)
                                title="Configurações"
                                style="background: transparent; border: none; cursor: pointer; font-size: 1.2rem; padding: 0; opacity: 0.7;"
                            >"⚙️"</button>
                        </div>
                        <div style="display: flex; flex-direction: row; gap: 5px; margin-left: 0px; margin-top: 5px;">
                            <button on:click=acao_sair style="background: #d32f2f; color: white; border: none; border-radius: 4px; padding: 5px 10px; cursor: pointer; font-size: 0.8rem;">"Sair"</button>
                            <button on:click=acao_resetar style="background: #455a64; color: white; border: none; border-radius: 4px; padding: 5px 10px; cursor: pointer; font-size: 0.8rem;">"Novo ID"</button>
                        </div>
                    </div>

                    // Placar e Status
                    <div style="text-align: right; display: flex; gap: 20px; align-items: center;">
                        <div style="text-align: right;">
                            <strong style="color: #ffeb3b; font-size: 1.1rem; text-shadow: 1px 1px 2px black; display: block;">
                                {move || {
                                    let status = state.status_jogo.get();
                                    let vez_id = state.turno_atual_id.get();
                                    let eu = state.meu_id.get();
                                    let nomes = state.mapa_nomes.get();
                                    let texto_vez = if vez_id == eu { "SUA VEZ!".to_string() } else {
                                        let nome = nomes.get(&vez_id).cloned().unwrap_or(format!("Jogador {}", vez_id));
                                        format!("Vez de {}", nome)
                                    };
                                    format!("{}. {}", status, texto_vez)
                                }}
                            </strong>
                        </div>

                        <Scoreboard
                            pontuacao_a=state.pontuacao_a
                            pontuacao_b=state.pontuacao_b
                            historico_a=state.historico_a
                            historico_b=state.historico_b
                            nomes=state.mapa_nomes
                            my_id=state.meu_id
                            on_click_expand=Callback::new(move |_| state.show_history.set(true))
                        />
                    </div>
                </div>

                // === GERENCIADORES INVISÍVEIS (Atalhos e Modais) ===
                <ShortcutManager
                    bindings=state.key_bindings
                    on_buy_deck=Callback::new(move |_| actions().comprar_monte())
                    on_discard=Callback::new(move |_| actions().descartar())
                    on_buy_trash=Callback::new(move |_| {
                        state.lixo_selecionado.set(true);
                        actions().confirmar_compra_lixo();
                    })
                    on_separate=Callback::new(move |_| actions().separar())
                    on_sort=Callback::new(move |_| actions().organizar_mao())
                    on_toggle_scoreboard=Callback::new(move |_| state.show_history.update(|v| *v = !*v))
                />

                <Show when=move || state.show_history.get() fallback=|| ()>
                    <ScoreHistoryModal
                        historico_a=state.historico_a
                        historico_b=state.historico_b
                        nomes=state.mapa_nomes
                        my_id=state.meu_id
                        scale=1.2
                        on_close=Callback::new(move |_| state.show_history.set(false))
                    />
                </Show>

                <SettingsModal
                    show=state.show_settings
                    current_theme_path=state.current_theme
                    card_scale=state.card_scale
                    volume=state.volume
                    key_bindings=state.key_bindings

                />

                // === ÁREA CENTRAL (Mesas e Board) ===
                <div style="flex: 1; display: flex; flex-direction: row; justify-content: space-between; align-items: flex-start; padding: 20px; gap: 20px; overflow-y: auto;">
                    // Mesa Esquerda (Time A se eu for par, B se eu for ímpar)
                    {move || {
                        let sou_time_a = state.meu_id.get().is_multiple_of(2);
                        let cb = if sou_time_a { Some(Callback::new(move |idx| actions().ajuntar(idx))) } else { None };
                        let titulo = if sou_time_a { "MEU TIME" } else { "TIME INIMIGO" };
                        view! {
                            <Table
                                titulo=titulo.to_string()
                                jogos=state.mesa_a
                                tres_vermelhos=state.tres_vermelhos_a
                                on_click=cb
                                theme=state.current_theme.get()
                                card_width=table_width
                                is_my_team=sou_time_a
                                highlighted_ids=state.highlighted_games
                            />
                        }
                    }}

                    // Centro (Lixo, Monte e Turnos)
                    <div style="display: flex; flex-direction: column; align-items: center; gap: 20px; flex-shrink: 0; margin-top: 40px;">
                        <Board
                            lixo=state.lixo_topo
                            lixo_selecionado=state.lixo_selecionado
                            on_click_deck=Some(Callback::new(move |_| actions().comprar_monte()))
                            on_click_trash=Some(Callback::new(move |_| actions().toggle_lixo_selecao()))
                            theme=state.current_theme.get()
                            card_width=board_width
                            qtd_monte=state.qtd_monte
                            qtd_lixo=state.qtd_lixo
                            verso_monte=state.verso_monte
                        />
                        <div style="background: rgba(0,0,0,0.2); padding: 10px; border-radius: 50%; border: 1px solid rgba(255,255,255,0.1);">
                            <TurnIndicator
                                my_id=state.meu_id
                                current_turn=state.turno_atual_id
                                names=state.mapa_nomes
                                cards_count=state.qtd_cartas_jogadores
                            />
                        </div>
                    </div>

                    // Mesa Direita
                    {move || {
                        let sou_time_b = !state.meu_id.get().is_multiple_of(2);
                        let cb = if sou_time_b { Some(Callback::new(move |idx| actions().ajuntar(idx))) } else { None };
                        let titulo = if sou_time_b { "MEU TIME" } else { "TIME INIMIGO" };
                        view! {
                            <Table
                                titulo=titulo.to_string()
                                jogos=state.mesa_b
                                tres_vermelhos=state.tres_vermelhos_b
                                on_click=cb
                                theme=state.current_theme.get()
                                card_width=table_width
                                is_my_team=sou_time_b
                                highlighted_ids=state.highlighted_games
                            />
                        }
                    }}
                </div>

                // === ÁREA INFERIOR (Mão e Controles) ===
                <div style="flex-shrink: 0; background: linear-gradient(to top, rgba(0,0,0,0.9) 20%, transparent); padding-bottom: 20px; position: relative; z-index: 10;">

                    // Área de "Jogos a Baixar" (Preparação)
                    <Show when=move || !state.jogos_preparados.get().is_empty() fallback=|| ()>
                        <div style="display: flex; justify-content: center; margin-bottom: 10px;">
                            <div style="background: rgba(0,0,0,0.5); padding: 10px; border-radius: 10px; border: 1px dashed #ffeb3b; text-align: center;">
                                <h4 style="margin: 0 0 10px 0; color: #ffeb3b; font-size: 12px;">"Jogos a Baixar"</h4>
                                <div style="display: flex; gap: 10px;">
                                    {move || state.jogos_preparados.get().into_iter().enumerate().map(|(idx, cartas)| {
                                        view! {
                                            <div on:click=move |_| actions().devolver(idx) style="cursor: pointer; display: flex; transform: scale(0.8);">
                                                {cartas.into_iter().map(|c| view! { <CardImage carta=c width="40px" theme=state.current_theme.get() /> }).collect::<Vec<_>>()}
                                            </div>
                                        }
                                    }).collect::<Vec<_>>()}
                                </div>
                                <button on:click=move |_| actions().confirmar_baixa() style="margin-top: 5px; background: #2e7d32; color: white; border: none; padding: 5px 15px; border-radius: 4px; cursor: pointer;">"Confirmar"</button>
                            </div>
                        </div>
                    </Show>

                    <div style="display: flex; align-items: flex-end; gap: 20px; width: 100%; overflow: hidden; padding: 0 20px;">
                        // Botões de Ação
                        <div style="flex-shrink: 0; margin-bottom: 20px;">
                            <GameControls
                                lixo_selecionado=state.lixo_selecionado
                                tem_jogos_preparados=Signal::derive(move || !state.jogos_preparados.get().is_empty())
                                on_descartar=Callback::new(move |_| actions().descartar())
                                on_separar=Callback::new(move |_| actions().separar())
                                on_ordenar=Callback::new(move |_| actions().organizar_mao())
                                on_confirmar_lixo=Callback::new(move |_| actions().confirmar_compra_lixo())
                                on_confirmar_baixa=Callback::new(move |_| actions().confirmar_baixa())
                                on_cancelar_lixo=Callback::new(move |_| actions().cancelar_lixo())
                            />
                        </div>

                        // Mão do Jogador
                        <div style="flex-grow: 1; min-width: 0;">
                            {move || {
                                let _mao = state.minha_mao.get(); // Trigger de update
                                view! {
                                    <Hand
                                        cartas=state.minha_mao
                                        card_width=hand_card_width
                                        theme=state.current_theme.get()
                                        selected_indices=state.selected_indices
                                    />
                                }
                            }}
                        </div>
                    </div>
                </div>

                // === ELEMENTOS INVISÍVEIS (Audio, Toast) ===
                <NotificationToast toasts=state.toasts />
                <audio node_ref=state.my_turn_audio_ref src="/assets/audio/my_turn_xylophone.wav" style="display: none;" />
                <audio node_ref=state.draw_audio_ref src="/assets/audio/draw1.ogg" style="display: none;" prop:preload="auto" />
            </div>
        </Show>
    }
}
