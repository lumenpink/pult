# Especificação: Sincronização dos Manifestos Cargo.toml das Crates com VERSION (Ticket #36)

- **Ticket:** #36
- **Norma:** GOST 1.1.0 (Perfil: `lumen.md`)
- **Status:** Implementado e Validado

## 1. Escopo e Motivação
Garantir que todos os subcrates pertencentes ao workspace do Pult (`crates/pult-core` e `crates/pult-tui`) mantenham paridade e sincronização estrita de versão com o arquivo `VERSION` raiz do repositório, gerenciado de forma preditiva pelo CCCP.

## 2. Requisitos Técnicos
1. **Consistência de Versão no Workspace:**
   - O arquivo `VERSION` na raiz do repositório define a versão canônica (ex.: `0.1.0-dev.239+20261029.0d68603`).
   - Os arquivos `crates/pult-core/Cargo.toml` e `crates/pult-tui/Cargo.toml` devem conter o campo `version` exatamente igual à versão contida no arquivo `VERSION` raiz.
2. **Identificação de CLI e TUI:**
   - A compilação da crate `pult-tui` expõe sua versão via CLI com fidelidade ao `VERSION` corrente, eliminando discrepâncias onde versões estáticas antigas (ex.: `dev.1`) eram reportadas por ferramentas de detecção e pela Jornaleira.
3. **Validação Automatizada:**
   - Um teste de integração deve verificar programaticamente que a versão lida de `VERSION` é idêntica à versão configurada nos manifestos do workspace.

## 3. Conclusão e Resultados
- Sincronização automatizada integrada com a nova capacidade do CCCP.
- Teste unitário `test_cargo_pkg_version_matches_canonical_root_version` aprovado.
- Quality Gate aprovado sem advertências (-D warnings).
