# Atalhos para o dia a dia. `make` sozinho lista os alvos.

# Encontra o cargo mesmo logo depois do bootstrap, antes de o shell recarregar o PATH.
export PATH := $(HOME)/.cargo/bin:$(PATH)

CARGO ?= cargo
ARQ   ?= exemplos/fatorial.mk
ARGS  ?=

.DEFAULT_GOAL := help
.PHONY: help bootstrap build release run tokens test fmt lint check clean

help: ## Lista os alvos disponíveis
	@grep -E '^[a-z]+:.*## ' $(MAKEFILE_LIST) | awk -F ':.*## ' '{printf "  make %-10s %s\n", $$1, $$2}'
	@echo
	@echo "  Variáveis: ARQ=<arquivo .mk> (padrão $(ARQ)), ARGS=<flags do monk>"
	@echo "  Exemplo:   make run ARQ=exemplos/escopos.mk ARGS=--tokens"

bootstrap: ## Instala Rust, rustfmt, clippy e o que mais faltar na máquina
	@sh scripts/bootstrap.sh

build: ## Compila em modo debug
	$(CARGO) build

release: ## Compila otimizado em target/release/monk
	$(CARGO) build --release

run: ## Roda o monk em ARQ, repassando ARGS
	@$(CARGO) run -q -- $(ARQ) $(ARGS)

tokens: ## Imprime os tokens de ARQ
	@$(CARGO) run -q -- $(ARQ) --tokens

test: ## Roda todos os testes
	$(CARGO) test

fmt: ## Formata o código
	$(CARGO) fmt

lint: ## Confere formatação e roda o clippy sem avisos
	$(CARGO) fmt --check
	$(CARGO) clippy --all-targets -- -D warnings

check: lint test ## Mesma verificação do CI (fmt, clippy e testes)

clean: ## Apaga os artefatos de compilação
	$(CARGO) clean
