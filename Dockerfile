# ==============================================================================
# ESTÁGIO 1: BUILDER (Compila o Rust e o Frontend)
# ==============================================================================
FROM rust:1.84-slim-bookworm as builder

# 1. Instala dependências do sistema necessárias para compilar
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    protobuf-compiler \
    && rm -rf /var/lib/apt/lists/*

# 2. Instala o Trunk (para o frontend) e adiciona o alvo WebAssembly
RUN cargo install trunk
RUN rustup target add wasm32-unknown-unknown

# 3. Define a pasta de trabalho
WORKDIR /app

# 4. Copia todo o código fonte do projeto para dentro do Docker
COPY . .

# 5. Compila o Frontend (Leptos/Wasm)
# O Trunk vai gerar os arquivos na pasta /app/buracao-web/dist
WORKDIR /app/buracao-web
RUN trunk build --release

# 6. Compila o Backend (Servidor)
# Voltamos para a raiz para o Cargo enxergar o workspace corretamente
WORKDIR /app
RUN cargo build --release --package buracao-server

# ==============================================================================
# ESTÁGIO 2: RUNNER (A imagem final leve que vai rodar no servidor)
# ==============================================================================
FROM debian:bookworm-slim

# 1. Instala dependências mínimas de runtime (SSL para HTTPS/WebSockets seguro)
RUN apt-get update && apt-get install -y libssl-dev ca-certificates && rm -rf /var/lib/apt/lists/*

# 2. Define a pasta de trabalho
WORKDIR /app

# 3. Copia o executável do servidor compilado no estágio anterior
COPY --from=builder /app/target/release/buracao-server /app/buracao-server

# 4. Copia o site compilado (pasta dist) para o local que o servidor espera (./dist)
COPY --from=builder /app/buracao-web/dist /app/dist

# 5. Expõe a porta 8080 (que configuramos no main.rs)
EXPOSE 8080

# 6. Comando para rodar o jogo
CMD ["./buracao-server"]
