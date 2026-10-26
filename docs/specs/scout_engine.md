# Especificação Técnica: Local Ollama Client & Scout Engine

## 1. Visão Geral e Propósito
O subsistema **Scout Engine** atua como segundo nível do funil de contexto do `pult`. Ele utiliza a GPU local (NVIDIA GeForce GTX 1050 com 4 GB de VRAM) para executar um modelo de linguagem local rápido (`qwen2.5-coder:3b` em quantização Q4_K_M) gerenciado pelo Ollama local.

O objetivo do Scout é cruzar a intenção digitada pela usuária (prompt/tarefa) com o sumário completo das assinaturas de AST extraídas pelo primeiro nível (`AstExtractor`), selecionando cirurgicamente apenas os 2 a 4 símbolos/traits/funções diretamente afetados. Esse bloco compacto (`injected_ast`) é então injetado no turno inicial do agente Antigravity, eliminando desperdício massivo de tokens e loops redundantes de descoberta no agente remoto.

## 2. Cliente Assíncrono do Ollama (`OllamaClient`)
O `OllamaClient` é construído com chamadas HTTP assíncronas utilizando `reqwest` e `tokio`.

### 2.1 Descoberta Inteligente de Conectividade
A resolução do endpoint base avalia ordenadamente:
1. Variável de ambiente `OLLAMA_HOST` (ex.: `http://127.0.0.1:11434` ou `tcp://localhost:11434`).
2. Loopback local padrão: `http://localhost:11434`.
3. Gateway de container padrão do Docker: `http://172.17.0.1:11434`.

Um método de verificação `check_health()` ou `discover_endpoint()` testa a rota `/api/tags` com timeout curto (ex.: 1 a 2 segundos) para validar a prontidão do daemon.

### 2.2 Endpoints da API Local
- `GET /api/tags`: Lista modelos instalados localmente.
- `GET /api/ps`: Monitora modelos carregados na VRAM da GPU, com informações de tamanho alocado (`size_vram`), parâmetros e expiração (`expires_at`).
- `POST /api/generate`: Gera inferência local. Suporta requisições com `stream: false` para respostas atômicas e streaming para monitoramento contínuo.

### 2.3 Modelo Padrão e Fallbacks
- Modelo preferencial canônico: `qwen2.5-coder:3b`.
- Fallbacks aceitáveis: `llama3.2:3b`, `llama3.2:1b`.

## 3. Motor de Filtragem Scout (`ScoutEngine`)
O `ScoutEngine` recebe:
1. A descrição da intenção da desenvolvedora (ex.: `"Implementar limitador de requisições em memória"`).
2. O catálogo de assinaturas de AST (`Vec<ExtractedAstFile>`).

### 3.1 Construção do Prompt de Filtragem
O prompt para o Scout é estruturado para forçar respostas estruturadas em bloco YAML:
```text
Você é o Scout, um assistente local de filtragem cirúrgica de código.
Analise a tarefa e o sumário de assinaturas da base de código abaixo.
Identifique EXCLUSIVAMENTE os 2 a 4 símbolos, funções, structs ou traits diretamente pertinentes para implementar a tarefa.
Responda APENAS com um bloco de código yaml no seguinte formato:

```yaml
injected_ast:
  - "caminho/arquivo: assinatura do simbolo"
```
```

### 3.2 Resposta e Estrutura de Domínio
O motor emite `ScoutResult`:
- `symbols: Vec<String>`: Lista de assinaturas selecionadas.
- `yaml_block: String`: Bloco formatado pronto para injeção no Markdown da sessão.
- `model_used: String`: Identificador do modelo local utilizado.
- `vram_allocated_bytes: Option<u64>`: Telemetria da VRAM obtida via `/api/ps`.

## 4. Tratamento de Erros e Suporte a Bypass
Em caso de falha de conexão ou ausência do serviço Ollama:
- O cliente retorna erro tipado `ScoutError::OllamaUnreachable`.
- O motor disponibiliza método `filter_symbols_with_bypass(...)` ou flag de fallback permitindo que o `pult` continue a operação sem abortar quando a usuária optar por bypass manual ou trabalho sem GPU local.
