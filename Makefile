# Soma - AI 短视频自动生成工具
# Makefile for development workflow

# ==================== Configuration ====================
BACKEND_PORT  := 8090
FRONTEND_PORT := 5273
CONFIG_FILE   := conf/config.toml

# Paths
WEB_DIR       := web
STORAGE_DIR   := storage
CONF_DIR      := conf
TAURI_DIR     := crates/soma-app

# Colors for output
COLOR_RESET   := \033[0m
COLOR_GREEN   := \033[32m
COLOR_YELLOW  := \033[33m
COLOR_CYAN    := \033[36m

# ==================== Targets ====================

.DEFAULT_GOAL := help

## help: Show all available commands
help: ## Show this help message
	@echo ""
	@echo "$(COLOR_CYAN)Soma - AI 短视频自动生成工具$(COLOR_RESET)"
	@echo "================================================"
	@echo ""
	@echo "$(COLOR_GREEN)Available commands:$(COLOR_RESET)"
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | \
		awk 'BEGIN {FS = ":.*?## "}; {printf "  $(COLOR_YELLOW)%-18s$(COLOR_RESET) %s\n", $$1, $$2}'
	@echo ""
	@echo "$(COLOR_CYAN)Ports:$(COLOR_RESET)  backend=$(BACKEND_PORT)  frontend=$(FRONTEND_PORT)"
	@echo ""

## install: Install all dependencies (Rust + Node.js)
install: install-backend install-frontend ## Install all dependencies

## install-backend: Install Rust dependencies
install-backend: ## Install Rust dependencies
	@echo "$(COLOR_GREEN)>>> Installing Rust dependencies...$(COLOR_RESET)"
	cargo fetch

## install-frontend: Install Node.js dependencies
install-frontend: ## Install frontend dependencies
	@echo "$(COLOR_GREEN)>>> Installing frontend dependencies...$(COLOR_RESET)"
	cd $(WEB_DIR) && npm install

## install-tauri: Install Tauri CLI
install-tauri: ## Install Tauri CLI
	@echo "$(COLOR_GREEN)>>> Installing Tauri CLI...$(COLOR_RESET)"
	cargo install tauri-cli

# ==================== Desktop App (Tauri) ====================

## dev-tauri: Start Tauri desktop app in development mode
dev-tauri: check-storage ## Start Tauri desktop app (dev mode)
	@echo "$(COLOR_GREEN)>>> Starting Soma desktop app...$(COLOR_RESET)"
	cargo tauri dev

## build-tauri: Build Tauri desktop app for production
build-tauri: ## Build desktop app for distribution
	@echo "$(COLOR_GREEN)>>> Building desktop app...$(COLOR_RESET)"
	cargo tauri build

## dev-tauri-debug: Start Tauri desktop app in debug mode (faster build)
dev-tauri-debug: check-storage ## Start Tauri desktop app (debug mode)
	@echo "$(COLOR_GREEN)>>> Starting Soma desktop app (debug)...$(COLOR_RESET)"
	cargo tauri dev --debug

# ==================== Web Server Mode ====================

## dev: Start both backend and frontend (web development mode)
dev: ## Start development servers (backend + frontend)
	@echo "$(COLOR_GREEN)>>> Starting Soma development environment...$(COLOR_RESET)"
	@echo "$(COLOR_CYAN)    Backend:  http://localhost:$(BACKEND_PORT)$(COLOR_RESET)"
	@echo "$(COLOR_CYAN)    Frontend: http://localhost:$(FRONTEND_PORT)$(COLOR_RESET)"
	@echo ""
	@$(MAKE) -j 2 dev-backend dev-frontend

## dev-backend: Start Rust backend only (development mode)
dev-backend: check-storage ## Start backend development server
	@echo "$(COLOR_GREEN)>>> Starting backend on port $(BACKEND_PORT)...$(COLOR_RESET)"
	cargo run -p soma-server

## dev-frontend: Start Vue frontend only (Vite dev server)
dev-frontend: ## Start frontend Vite dev server
	@echo "$(COLOR_GREEN)>>> Starting frontend on port $(FRONTEND_PORT)...$(COLOR_RESET)"
	cd $(WEB_DIR) && npm run dev

## build: Build both backend (release) and frontend (production)
build: build-backend build-frontend ## Build all for production

## build-backend: Build Rust backend in release mode
build-backend: ## Build backend (release)
	@echo "$(COLOR_GREEN)>>> Building backend (release)...$(COLOR_RESET)"
	cargo build --release -p soma-server

## build-frontend: Build Vue frontend for production
build-frontend: ## Build frontend for production
	@echo "$(COLOR_GREEN)>>> Building frontend (production)...$(COLOR_RESET)"
	cd $(WEB_DIR) && npm run build

## prod: Build frontend then run backend in release mode (serves SPA)
prod: build-frontend ## Build frontend, then run backend (production)
	@echo "$(COLOR_GREEN)>>> Starting production server on port $(BACKEND_PORT)...$(COLOR_RESET)"
	cargo run --release -p soma-server

## run: Quick alias for cargo run (backend only, debug mode)
run: ## Quick start backend in debug mode
	cargo run -p soma-server

## clean: Clean all build artifacts
clean: ## Clean all build artifacts
	@echo "$(COLOR_YELLOW)>>> Cleaning build artifacts...$(COLOR_RESET)"
	cargo clean
	cd $(WEB_DIR) && rm -rf dist node_modules/.vite
	@echo "$(COLOR_GREEN)Done.$(COLOR_RESET)"

## check: Check Rust compilation (no binary output)
check: ## Check Rust compilation
	cargo check

## test: Run Rust tests
test: ## Run Rust tests
	cargo test

## fmt: Format Rust code
fmt: ## Format Rust code
	cargo fmt

## lint: Run clippy linter
lint: ## Run clippy linter
	cargo clippy

# ==================== Internal helpers ====================

check-storage:
	@mkdir -p $(STORAGE_DIR)/songs $(STORAGE_DIR)/fonts $(STORAGE_DIR)/cache_videos

.PHONY: help install install-backend install-frontend install-tauri \
		dev-tauri build-tauri dev-tauri-debug \
		dev dev-backend dev-frontend \
		build build-backend build-frontend prod run \
		clean check test fmt lint check-storage


ifneq ($(filter git,$(MAKECMDGOALS)),)
  GIT_MSG_ARGS := $(wordlist 2,$(words $(MAKECMDGOALS)),$(MAKECMDGOALS))
  $(foreach _g,$(GIT_MSG_ARGS),$(eval $(_g):;@:))
endif

.PHONY: git

git:
	@set -e; \
	msg=''; \
	if [ -n "$(strip $(MSG))" ]; then \
		msg='$(subst ','\'',$(MSG))'; \
	elif [ -n "$(strip $(GIT_MSG_ARGS))" ]; then \
		msg='$(subst ','\'',$(GIT_MSG_ARGS))'; \
	else \
		printf 'input commit message: '; read -r msg; \
	fi; \
	git add . && \
	git commit -a -m "$$msg" && \
	git pull && \
	git push && \
	echo git commit and push success
