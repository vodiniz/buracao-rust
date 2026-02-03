use crate::utils::assets::get_card_path;
use buracao_core::baralho::{Carta, Naipe, Valor};

pub fn carta_para_asset(carta: &Carta) -> String {
    let valor_str = match carta.valor {
        Valor::As => "a",
        Valor::Dois => "2",
        Valor::Tres => "3",
        Valor::Quatro => "4",
        Valor::Cinco => "5",
        Valor::Seis => "6",
        Valor::Sete => "7",
        Valor::Oito => "8",
        Valor::Nove => "9",
        Valor::Dez => "10", // Mudamos de "t" para "10"
        Valor::Valete => "j",
        Valor::Dama => "q",
        Valor::Rei => "k",
        Valor::Joker => return "joker_r".to_string(),
    };

    let naipe_str = match carta.naipe {
        Naipe::Paus => "c",
        Naipe::Ouros => "d",
        Naipe::Copas => "h",
        Naipe::Espadas => "s",
        Naipe::Nenhum => "joker",
    };

    if carta.valor == Valor::Joker {
        return "joker_b".to_string();
    }

    format!("{}_{}", naipe_str, valor_str)
}

pub fn carta_para_asset_path(carta: &Carta) -> String {
    let id_arquivo = match carta.valor {
        Valor::Joker => "joker_r".to_string(), // Coringa Vermelho
        _ => {
            let valor_str = match carta.valor {
                Valor::As => "a",
                Valor::Valete => "j",
                Valor::Dama => "q",
                Valor::Rei => "k",
                Valor::Dez => "10",
                v => {
                    // Para valores numéricos, usamos a representação simples
                    // Assumindo que seu enum Valor possa ser convertido ou comparado
                    match v {
                        Valor::Dois => "2",
                        Valor::Tres => "3",
                        Valor::Quatro => "4",
                        Valor::Cinco => "5",
                        Valor::Seis => "6",
                        Valor::Sete => "7",
                        Valor::Oito => "8",
                        Valor::Nove => "9",
                        _ => "unknown",
                    }
                }
            };

            let naipe_str = match carta.naipe {
                Naipe::Paus => "c",
                Naipe::Ouros => "d",
                Naipe::Copas => "h",
                Naipe::Espadas => "s",
                Naipe::Nenhum => "joker",
            };

            format!("{}_{}", naipe_str, valor_str)
        }
    };

    // O PULO DO GATO: Chamar a função que coloca o path e o .png
    get_card_path(&id_arquivo)
}
