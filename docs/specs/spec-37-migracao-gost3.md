# Especificação — Ticket #37: Migração para GOST v3.0 e Kollontai 2.1

## 1. Contexto e Motivação
O repositório `pult` utiliza o padrão GOST v1.1.0 (`GOSTVER` na raiz apontando para o arquivo legado `profiles/lumen.md`).
A infraestrutura Gosplan/Tulipa padronizou a metanorma no GOST v3.0, onde a autoridade normativa é estritamente compilada em `.gost/` e a orquestração de entregas é declarada em `.gost/delivery.toml`.

## 2. Escopo Técnico
1. Executar `cccp gost setup` para compilar o perfil Lilya v2.1.0 e as 24 regras da Kollontai v2.1.0.
2. Purgar o arquivo plano `GOSTVER` da raiz.
3. Configurar `.gost/delivery.toml` com frentes de entrega aplicáveis (`binaries` e `sites`).
4. Ajustar conformidade com as regras normativas (`cccp check --strict`).

## 3. Critérios de Aceite
- Arquivo `GOSTVER` ausente na raiz.
- Diretório `.gost/` contendo `GOST.md`, `rules.toml`, `metadata.json` e `delivery.toml`.
- `cccp check --strict` aprovado com 100% de conformidade.

## 4. Laudo de Resolução (Fase 9)
- **Status:** Concluído com 100% de conformidade GOST.
- **Data de Fechamento:** 2026-11-18
- **Implementação:**
  - `GOSTVER` legado na raiz purgado.
  - `.gost/` compilado com 24 regras do perfil Lilya v2.1.0 e Kollontai v2.1.0.
  - `.gost/delivery.toml` configurado para `binaries` (Banca) e `sites` (Cloudflare Pages).
  - `.cccprc` alinhado à forja canônica Gambi (`forgejo_url = https://gambi.2lp.in`).
  - `README.md` integrado com assertiva de identidade transfeminista (`lumen.pink`) e termos colaborativos.
  - Expurgo de termos proibidos em especificações (`docs/specs/*.md`) e adequação de contingência i18n em `site/index.html`.
  - Hooks do maestro instalados via `cccp hook -i -f`.
- **Validação:** Testes unitários do crate `pult-core` aprovados, `cargo fmt --check` limpo, `cargo clippy` sem avisos e `cccp check --strict` com 24 regras conformes (100%).

