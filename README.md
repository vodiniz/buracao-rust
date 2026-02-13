# Buracão Web - Implementação em Rust

Este repositório contém uma implementação completa do jogo de cartas **Buracão** (uma variação do Buraco/Canastra sem morto), desenvolvida inteiramente em **Rust**. O projeto é um *monorepo* que divide a lógica do jogo, o servidor WebSocket e o frontend WebAssembly.

---

## 🏗️ Estrutura do Projeto

O projeto é organizado como um Rust Workspace com três pacotes principais:

### 1. `buracao-core`

O núcleo do sistema. Contém toda a lógica de negócios, regras, validações e estruturas de dados do jogo.

- **Responsabilidade:** Código Rust puro, sem dependências de rede ou interface gráfica.
- **Bibliotecas:**
  - `serde`: Para serialização/deserialização dos estados do jogo (comunicação JSON).
  - `rand`: Para embaralhamento das cartas.

---

### 2. `buracao-server`

O servidor backend que gerencia as salas (lobbies) e o estado multiplayer.

- **Responsabilidade:**  
  Gerencia conexões WebSocket, mantém o estado das salas em memória e faz o *broadcast* das mensagens para os jogadores.
- **Bibliotecas:**
  - `warp`: Framework web leve e rápido para lidar com HTTP e WebSockets.
  - `tokio`: Runtime assíncrono para lidar com múltiplas conexões simultâneas.
  - `futures`: Para manipulação de streams assíncronas.

---

### 3. `buracao-web`

O cliente frontend que roda no navegador do usuário.

- **Responsabilidade:**  
  Interface gráfica interativa compilada para WebAssembly (WASM).
- **Bibliotecas:**
  - `leptos`: Framework reativo moderno para construir interfaces web em Rust (similar a React/Solid).
  - `gloo-net`: Utilitários para facilitar o uso de WebSockets no navegador.
  - `wasm-bindgen`: Ponte de comunicação entre Rust e JavaScript.

---

## 🚀 Como Compilar e Rodar

### 📦 Pré-requisitos

1. **Rust:** Tenha o [Rust instalado](https://www.rust-lang.org/tools/install).
2. **Trunk:** Ferramenta de build para WASM. Instale com:

```bash
cargo install trunk
```

3. **Target WASM:** Adicione o alvo de compilação:

```bash
rustup target add wasm32-unknown-unknown
```

---

## ▶️ Passo a Passo

Você precisará de dois terminais abertos.

### 🖥️ Terminal 1 - Servidor (Backend)

```bash
cd buracao-server
cargo run
```

O servidor iniciará na porta `8080` (ex: `0.0.0.0:8080`).

---

### 🌐 Terminal 2 - Cliente (Frontend)

```bash
cd buracao-web
trunk serve
```

O Trunk irá compilar o projeto e servir em:

```
http://127.0.0.1:8080
```

(ou porta similar indicada no terminal)

Abra múltiplas abas (ou janelas anônimas) para simular os jogadores.

---

# 🃏 Regras do Jogo

As regras abaixo estão organizadas cronologicamente, desde a preparação até a pontuação final.

---

## 1️⃣ Preparação

- **Baralho:** O jogo é jogado com 2 baralhos completos, incluindo os Coringões (Jokers).
- **Mão:** São distribuídas 15 cartas para cada jogador.
- **Sem Morto:** Nesta modalidade, não existe morto.
- **Rodízio:** A pessoa que começa a partida muda a cada rodada.

---

## 2️⃣ Valor e Funções das Cartas

### 🔹 Sequências

- Os jogos consistem apenas de sequências do mesmo naipe.
- As sequências começam obrigatoriamente no 4 e vão até o Ás (A).
- **Exceção:** É permitido um jogo contendo apenas Ases (ex: trinca de A).

### 🔹 Coringas

- O **Coringão (Joker)** e o **Coringuinha (2)** substituem qualquer carta.
- Só é permitido **um coringa (Joker ou 2)** por jogo baixado.

### 🔹 Cartas Especiais

- **3 Preto:**  
  É inútil, não vale pontos e serve apenas para descartar e travar o lixo.

- **3 Vermelho:**  
  Se receber, deve colocar na mesa e comprar outra carta imediatamente (seguindo a ordem do turno).

---

## 3️⃣ Fluxo do Turno

Em cada turno, você deve:

1. **Comprar** (Monte ou Lixo)  
2. **Baixar Jogos** (Opcional)  
3. **Descartar**

---

### 🗑️ Pegar o Lixo

- Você só pode pegar o lixo se utilizar a carta do topo imediatamente para:
  - Baixar um jogo novo, ou
  - Ajuntar em um existente (totalizando 3 cartas com a do topo).

- **Trava:**  
  3 Preto, Coringuinha e Coringão no topo do lixo impedem a compra do lixo pela próxima pessoa.

---

### 🃏 Baixar Jogos

- Para baixar um jogo novo, precisa de no mínimo **3 cartas**.

#### Pontuação de Saída (Primeira descida):

- Se o time tem menos de 2500 pontos:  
  ➜ Precisa de **80 pontos** para descer.

- Se o time tem 2500 pontos ou mais:  
  ➜ Precisa de **100 pontos** para descer.

---

## 4️⃣ Tipos de Jogos (Canastras)

### ⭐ Real (Limpa)

- 7 cartas em sequência ordenadas **sem coringa (2)**.
- Pode conter Coringão.
- Vale **300 pontos**.

### ⚠️ Suja

- Jogo de 7 cartas contendo um coringa (2).
- Vale **100 pontos**.

---

## 5️⃣ Encerramento (Batida) e Fim do Monte

### 🏁 Condição de Vitória

- Para bater (zerar a mão), você **PRECISA ter pelo menos uma Real**.
- Não é permitido bater pegando o lixo.

### 📦 Fim do Monte

- Se as cartas de compra acabarem:
  - Ninguém é penalizado pelas cartas na mão.
  - Haverá mais uma rodada onde a pessoa seguinte pode tentar jogar com o lixo (apenas se conseguir descer).

---

## 6️⃣ Pontuação Final

### 💰 Valor das Cartas

- Todas as cartas valem **10 pontos**.
- Coringão (Joker) vale **20 pontos**.

### 🎯 Batida

- O jogador que bate ganha **100 pontos extras**.

### ❌ Penalidade

- Se alguém bater, os adversários perdem os pontos equivalentes à soma das cartas que sobraram em suas mãos.

### ❤️ 3 Vermelho

- Se o time tem Real:  
  ➜ Vale **+100 pontos**

- Se o time NÃO tem Real:  
  ➜ Vale **-100 pontos**

---
