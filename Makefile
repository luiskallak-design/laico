# --- ARCHIA_OS: MAKEFILE ORQUESTRADOR CENTRAL (RUST EDITION) ---

# Nome do binário final instalado no sistema
BIN_NAME = laico-boot
TARGET_BIN = target/release/archia

# Diretório padrão local de destino do usuário (.cargo/bin)
CARGO_BIN_DIR = $(HOME)/.cargo/bin

# Comando padrão: Limpa o cache anterior e compila em modo otimizado
all: clean build
	@echo "⌬ [SISTEMA]: Compilação finalizada com sucesso."

# Compila o projeto em Rust usando o perfil de Release (Otimizado)
build:
	@echo "⌬ [BUILD]: Compilando módulo Archia via Cargo..."
	cargo build --release

# Instala o binário compilado direto no diretório local do usuário
install:
	@echo "⌬ [SISTEMA]: Instalando $(BIN_NAME) em $(CARGO_BIN_DIR)..."
	@mkdir -p $(CARGO_BIN_DIR)
	cp $(TARGET_BIN) $(CARGO_BIN_DIR)/$(BIN_NAME)
	@echo "⌬ [SISTEMA]: Instalação concluída."

# Executa a atualização forçada reiniciando o launcher no terminal
deploy: install
	@echo "⌬ [SISTEMA]: Atualizando interface em execução..."
	-killall $(BIN_NAME) 2>/dev/null || true
	$(BIN_NAME)

# --- LIMPEZA TÁTICA ---
clean:
	@echo "⌬ [SISTEMA]: Limpando cache de compilação antigo..."
	cargo clean

# --- MANUAL DE COMANDOS ---
help:
	@echo "Comandos disponíveis no Makefile:"
	@echo "  make         - Limpa o cache e compila o código em modo Release"
	@echo "  make build   - Compila o binário otimizado sem limpar o cache"
	@echo "  make install - Copia o binário pronto para ~/.cargo/bin/$(BIN_NAME)"
	@echo "  make deploy  - Instala e força o reinício automático do launcher"
	@echo "  make clean   - Remove a pasta 'target' e limpa o ambiente Rust"
