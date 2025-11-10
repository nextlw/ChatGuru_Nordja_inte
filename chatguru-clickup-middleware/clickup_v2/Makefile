.PHONY: help test test-unit test-integration test-all coverage bench fmt lint clean docs build release

# Cores para output
RED := \033[0;31m
GREEN := \033[0;32m
YELLOW := \033[0;33m
NC := \033[0m # No Color

help: ## Mostra esta mensagem de ajuda
	@echo "$(GREEN)ClickUp v2 Rust Crate - Comandos disponíveis:$(NC)"
	@echo ""
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "$(YELLOW)%-20s$(NC) %s\n", $$1, $$2}'

test: ## Executa todos os testes
	@echo "$(GREEN)Executando todos os testes...$(NC)"
	@export CLICKUP_CLIENT_ID=test_client_id && \
	export CLICKUP_CLIENT_SECRET=test_client_secret && \
	cargo test --verbose

test-unit: ## Executa apenas testes unitários
	@echo "$(GREEN)Executando testes unitários...$(NC)"
	@export CLICKUP_CLIENT_ID=test_client_id && \
	export CLICKUP_CLIENT_SECRET=test_client_secret && \
	cargo test --lib --verbose

test-integration: ## Executa apenas testes de integração
	@echo "$(GREEN)Executando testes de integração...$(NC)"
	@export CLICKUP_CLIENT_ID=test_client_id && \
	export CLICKUP_CLIENT_SECRET=test_client_secret && \
	cargo test --test '*' --verbose

test-doc: ## Executa testes de documentação
	@echo "$(GREEN)Executando testes de documentação...$(NC)"
	@export CLICKUP_CLIENT_ID=test_client_id && \
	export CLICKUP_CLIENT_SECRET=test_client_secret && \
	cargo test --doc --verbose

test-all: test test-doc ## Executa todos os testes incluindo documentação

coverage: ## Gera relatório de cobertura de código
	@echo "$(GREEN)Gerando relatório de cobertura...$(NC)"
	@export CLICKUP_CLIENT_ID=test_client_id && \
	export CLICKUP_CLIENT_SECRET=test_client_secret && \
	cargo tarpaulin --verbose --all-features --workspace --timeout 120 --out Html --output-dir ./coverage

coverage-open: coverage ## Gera e abre o relatório de cobertura no navegador
	@echo "$(GREEN)Abrindo relatório de cobertura...$(NC)"
	@open ./coverage/tarpaulin-report.html || xdg-open ./coverage/tarpaulin-report.html

bench: ## Executa benchmarks
	@echo "$(GREEN)Executando benchmarks...$(NC)"
	@export CLICKUP_CLIENT_ID=test_client_id && \
	export CLICKUP_CLIENT_SECRET=test_client_secret && \
	cargo bench --verbose

bench-save: ## Executa benchmarks e salva baseline
	@echo "$(GREEN)Executando benchmarks e salvando baseline...$(NC)"
	@export CLICKUP_CLIENT_ID=test_client_id && \
	export CLICKUP_CLIENT_SECRET=test_client_secret && \
	cargo bench --verbose -- --save-baseline main

bench-compare: ## Compara benchmarks com baseline
	@echo "$(GREEN)Comparando benchmarks com baseline...$(NC)"
	@export CLICKUP_CLIENT_ID=test_client_id && \
	export CLICKUP_CLIENT_SECRET=test_client_secret && \
	cargo bench --verbose -- --baseline main

fmt: ## Formata o código
	@echo "$(GREEN)Formatando código...$(NC)"
	@cargo fmt

fmt-check: ## Verifica formatação do código
	@echo "$(GREEN)Verificando formatação...$(NC)"
	@cargo fmt -- --check

lint: ## Executa clippy para análise estática
	@echo "$(GREEN)Executando análise estática com clippy...$(NC)"
	@cargo clippy -- -D warnings

lint-fix: ## Executa clippy e aplica correções automáticas
	@echo "$(GREEN)Aplicando correções do clippy...$(NC)"
	@cargo clippy --fix -- -D warnings

clean: ## Limpa arquivos de build e cache
	@echo "$(GREEN)Limpando arquivos de build...$(NC)"
	@cargo clean
	@rm -rf coverage/
	@rm -rf target/

docs: ## Gera documentação
	@echo "$(GREEN)Gerando documentação...$(NC)"
	@cargo doc --no-deps --open

docs-all: ## Gera documentação incluindo dependências
	@echo "$(GREEN)Gerando documentação completa...$(NC)"
	@cargo doc --open

build: ## Compila o projeto
	@echo "$(GREEN)Compilando projeto...$(NC)"
	@cargo build --verbose

build-release: ## Compila o projeto em modo release
	@echo "$(GREEN)Compilando em modo release...$(NC)"
	@cargo build --release --verbose

release: test lint fmt-check ## Prepara para release (testa, analisa e verifica formatação)
	@echo "$(GREEN)Projeto pronto para release!$(NC)"

check: ## Verifica se o projeto compila
	@echo "$(GREEN)Verificando compilação...$(NC)"
	@cargo check --all-targets --all-features

watch: ## Observa mudanças e re-executa testes
	@echo "$(GREEN)Observando mudanças...$(NC)"
	@cargo watch -x test

install-tools: ## Instala ferramentas de desenvolvimento
	@echo "$(GREEN)Instalando ferramentas de desenvolvimento...$(NC)"
	@cargo install cargo-tarpaulin || true
	@cargo install cargo-watch || true
	@cargo install cargo-edit || true
	@cargo install cargo-outdated || true

outdated: ## Verifica dependências desatualizadas
	@echo "$(GREEN)Verificando dependências desatualizadas...$(NC)"
	@cargo outdated

update: ## Atualiza dependências
	@echo "$(GREEN)Atualizando dependências...$(NC)"
	@cargo update

audit: ## Verifica vulnerabilidades de segurança
	@echo "$(GREEN)Verificando vulnerabilidades...$(NC)"
	@cargo audit

test-env: ## Cria arquivo .env de teste
	@echo "$(GREEN)Criando arquivo .env de teste...$(NC)"
	@echo "CLICKUP_CLIENT_ID=test_client_id" > .env.test
	@echo "CLICKUP_CLIENT_SECRET=test_client_secret" >> .env.test
	@echo "CLICKUP_REDIRECT_URI=http://localhost:8888/callback" >> .env.test
	@echo "$(YELLOW)Arquivo .env.test criado!$(NC)"

pre-commit: fmt lint test ## Executa verificações antes do commit
	@echo "$(GREEN)Todas as verificações passaram!$(NC)"

ci: test-all lint fmt-check ## Executa pipeline CI localmente
	@echo "$(GREEN)Pipeline CI executado com sucesso!$(NC)"

# Targets específicos para desenvolvimento
dev-setup: install-tools test-env ## Configura ambiente de desenvolvimento
	@echo "$(GREEN)Ambiente de desenvolvimento configurado!$(NC)"

# Target padrão
all: build test lint ## Compila, testa e analisa o código