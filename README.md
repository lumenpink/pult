# ☭ PULT (Пульт)

Estação de trabalho TUI soberana para terminal (Rust), mediadora de contexto AST/Scout e guardiã do ciclo DD-TDD (GOST/CCCP) com a Antigravity CLI.

## 📋 Conformidade GOST

Este projeto adota a norma **[GOST](https://codeberg.org/lumenpink/gost)** (Gosplan Standard for Technical Software).
- **Versão GOST:** `1.1.0`
- **Linguagens declaradas:** `rust`
- **Licença:** `GPL-3.0-or-later`

## 🏗️ Arquitetura do Workspace

O projeto é estruturado em dois crates desacoplados:
- **`crates/pult-core`:** Biblioteca de domínio, extratores de AST (Rust e poliglota), cliente Scout (Ollama/GPU), persistência de sessões markdown e driver CCCP/Git.
- **`crates/pult-tui`:** Interface de terminal em tela dividida (Ratatui + Crossterm + `tui-textarea`), gerenciamento de eventos de teclado e barra de status do Gosplan.

## 🚀 Como Iniciar

```bash
# Verificar conformidade com o Gosplan
cccp check --strict

# Executar suíte de testes
cargo test

# Executar a interface TUI
cargo run -p pult-tui
```

## 📄 Licença

Distribuído sob a licença [GPL-3.0-or-later](./LICENSE).
