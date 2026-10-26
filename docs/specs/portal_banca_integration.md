# Especificação: Portal pult.2lp.in e Integração com a Banca

- **Ticket:** #12
- **Norma:** GOST 1.1.0 (Perfil: `lumen.md`)
- **Status:** Aprovado

## 1. Escopo e Objetivos
1. **Portal Web Oficial (`pult.2lp.in`):**
   - Página estática anti-design / construtivista moderna em `site/index.html` com folha de estilo `site/assets/style.css`.
   - Terminal Mockup interativo em CSS simulando a interface desacoplada do Pult.
   - Centro de instalação (curl, pacotes binários na Banca, Cargo).
   - Cabeçalhos de segurança em `site/_headers` e roteamento em `wrangler.jsonc`.
   - Script de instalação portátil `install.sh` (e `site/install.sh`).

2. **Esteira CI/CD Multiarch & Deploy Banca (`.forgejo/workflows/ci.yaml`):**
   - Compilação dos alvos `x86_64-unknown-linux-musl`, `aarch64-unknown-linux-musl` e `x86_64-pc-windows-gnu`.
   - Empacotamento em `dist/packages/` com checksums SHA-256.
   - Upload de artefatos para o Cloudflare R2 (`twolp-bucket/downloads/pult/${CHANNEL}/...`).
   - Rotação do manifesto na Banca (`site/downloads/pult/index.json`) e atualização do `catalog.json`.
   - Deploy do portal em Cloudflare Pages.

3. **Catálogo da Banca (`banca`):**
   - Entrada oficial em `data/projects.json`.
   - Suporte no script `scripts/rotate-downloads.py`.
   - Banner anti-design `site/assets/banners/pult-banner.png`.
   - Manifesto inicial `site/downloads/pult/index.json`.
