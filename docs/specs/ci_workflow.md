# ☭ Especificação Normativa da Esteira de CI / CD (GOST)

**Ticket:** #11  
**Status:** Aprovado  

## 1. Contrato de Execução
O Forgejo Actions opera através de runners registrados sob a etiqueta `ubuntu-latest`.
Para garantir hermeticidade e isolamento de dependências, todos os jobs de checagem e teste de Rust devem executar dentro do container padrão do ecossistema: `gambi.2lp.in/lumen/gost-ci-cross:latest`.

## 2. Requisitos dos Jobs
1. **`check-and-test`**:
   - `runs-on`: `ubuntu-latest`
   - `container.image`: `gambi.2lp.in/lumen/gost-ci-cross:latest`
   - Passos obrigatórios:
     - `actions/checkout@v4` (com `fetch-depth: 0`)
     - `cargo check`
     - `cargo test`
     - `cccp check --strict`
2. **`deploy-site`**:
   - `runs-on`: `ubuntu-latest`
   - `needs`: `[check-and-test]`
   - Execução restrita à branch `main` em eventos de push
   - Publicação de `./site` no Cloudflare Workers (`pult.2lp.in`) via `wrangler deploy`.
