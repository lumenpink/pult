# Especificação Técnica: Desacoplamento do Parser de Auditoria de Qualidade e Tolerância em Testes (Ticket #33)

- **Ticket:** #33
- **Norma:** GOST 1.1.0 (Perfil: `lumen.md`)
- **Status:** Aprovado

## 1. Escopo e Motivação
A execução de testes de integração do `CccpGitDriver` (`test_mutation_check_strict_execution`) invocava incondicionalmente o binário `cccp check --strict`. Em ambientes de CI desprovidos do binário no `PATH`, o teste entrava em pânico imediato por `NotFound` (`Io(Os { code: 2 })`).
Além disso, a lógica de interpretação das métricas da auditoria estrita do Gosplan (`Total de verificações`, `Conformes`, `Avisos`, `Falhas` e `Índice de Conformidade Gosplan`) estava acoplada dentro do método `check_strict()`, impossibilitando testes unitários determinísticos e isolados.

Esta especificação define o desacoplamento da lógica de parsing para uma função pura testável, bem como o tratamento resiliente no teste de integração quando o binário não estiver presente no sistema.

## 2. Assinatura e Comportamento da Função Pura
Extrair e exportar a função pura em `pult_core::driver`:

```rust
pub fn parse_quality_audit(stdout: &str, status_success: bool) -> QualityAudit
```

### Regras de Parsing:
1. Linha com métricas divididas por pipe (`|`):
   - `Total de verificações:` -> converte para `u32` (default 0).
   - `Conformes:` -> converte para `u32` (default 0).
   - `Avisos:` -> converte para `u32` (default 0).
   - `Falhas:` -> converte para `u32` (default 0).
2. Linha de índice:
   - Prefixo `Índice de Conformidade Gosplan:` -> captura string aparada (ex.: `"100%"`). Default `"0%"`.
3. Cálculo do campo `passed`:
   - `passed = status_success && failures == 0`
4. O método `CccpGitDriver::check_strict()` passa a delegar o parsing para `parse_quality_audit(&stdout, output.status.success())`.

## 3. Resiliência nos Testes de Integração
- No teste `test_mutation_check_strict_execution`:
  - Se a execução de `driver.check_strict()` retornar `Err(DriverError::Io(err))` com `err.kind() == std::io::ErrorKind::NotFound`, a ausência do binário em ambientes mínimos de CI é tolerada com log/aviso informativo, sem falhar a suíte de testes.
  - Se for bem-sucedida, valida as asserções de sanidade do `QualityAudit`.
  - Quaisquer outros erros não esperados continuam a causar falha explícita.
- A cobertura exaustiva e determinística das regras de negócio do `QualityAudit` passa a ser garantida por testes unitários dedicados de `parse_quality_audit`.
