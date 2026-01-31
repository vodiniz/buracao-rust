use rand::seq::SliceRandom;
use serde::{Deserialize, Serialize};
use std::fmt;

// --- ESTRUTURA DAS CARTAS ---

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, PartialOrd, Ord, Hash)]
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

impl fmt::Display for Naipe {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let simbolo = match self {
            Naipe::Copas => "♥️",
            Naipe::Ouros => "♦️",
            Naipe::Paus => "♣️",
            Naipe::Espadas => "♠️",
            // Para o Joker que não tem naipe, não imprimimos nada
            Naipe::Nenhum => "",
        };
        write!(f, "{}", simbolo)
    }
}

impl fmt::Display for Valor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let val_str = match self {
            Valor::As => "A",
            Valor::Dois => "2", // Lembre-se que o 2 é coringuinha
            Valor::Tres => "3",
            Valor::Quatro => "4",
            Valor::Cinco => "5",
            Valor::Seis => "6",
            Valor::Sete => "7",
            Valor::Oito => "8",
            Valor::Nove => "9",
            Valor::Dez => "10",
            Valor::Valete => "J",
            Valor::Dama => "Q",
            Valor::Rei => "K",
            Valor::Joker => "🃏", // Usamos um emoji de carta para o Coringão
        };
        write!(f, "{}", val_str)
    }
}

// O Display da Carta junta os dois anteriores
impl fmt::Display for Carta {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Se for Joker, o naipe será "Nenhum" (vazio), imprimindo só 🃏
        // Se for carta normal, imprime Valor + Naipe (ex: A♥️)
        write!(f, "{}{}", self.valor, self.naipe)
    }
}

impl Valor {
    // Retorna um número para cálculo de distância (ex: As=1, Dois=2 ... Rei=13)
    pub fn indice_sequencia(&self) -> u8 {
        match self {
            Valor::As => 1,
            Valor::Dois => 2,
            Valor::Tres => 3,
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
            Valor::Joker => 0, // Joker não tem índice fixo
        }
    }
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

    pub fn eh_tres_preto(&self) -> bool {
        self.valor == Valor::Tres && (self.naipe == Naipe::Espadas || self.naipe == Naipe::Paus)
    }

    pub fn trava_o_lixo(&self) -> bool {
        self.eh_tres_preto() || self.valor == Valor::Dois || self.valor == Valor::Joker
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

    pub fn eh_joker(&self) -> bool {
        matches!(self.valor, Valor::Joker)
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
