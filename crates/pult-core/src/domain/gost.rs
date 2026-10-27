use crate::domain::error::SessionError;
use serde::{Deserialize, Serialize};

/// As 9 Fases Normativas do ciclo GOST DD-TDD.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum GostPhase {
    /// Fase 1: Formulação e discussão do escopo (sem commits no git).
    Ticket = 1,
    /// Fase 2: Criação da branch dedicada (`<tipo>/<id>-<slug>`).
    NewBranch = 2,
    /// Fase 3: Especificação de contratos, tipos e markdown (veto a código de produção).
    SpecContract = 3,
    /// Fase 4: Red test comprovando falha (`cargo test` exit != 0).
    RedTest = 4,
    /// Fase 5: Implementação mínima funcional para teste passar (`cargo test` exit == 0).
    GreenCode = 5,
    /// Fase 6: Polimento e Zero Warnings (`cargo clippy -- -D warnings`).
    Refactor = 6,
    /// Fase 7: Fortificação de asserções e eliminação de mutantes sobreviventes.
    MutationTest = 7,
    /// Fase 8: Quality Gate e auditoria local (cobertura >= 90%, cargo fmt).
    QualityGate = 8,
    /// Fase 9: Fechamento com crônica completa e tag `(closes #<id>)`.
    CloseTicket = 9,
}

impl GostPhase {
    /// Retorna o número ordinal da fase (1 a 9).
    pub fn number(&self) -> u8 {
        *self as u8
    }

    /// Retorna o nome canônico em português da fase.
    pub fn name(&self) -> &'static str {
        match self {
            Self::Ticket => "Ticket / Issue",
            Self::NewBranch => "New Branch",
            Self::SpecContract => "Doc / Spec Contract",
            Self::RedTest => "Red Test",
            Self::GreenCode => "Green Code",
            Self::Refactor => "Refactor",
            Self::MutationTest => "Mutation Test",
            Self::QualityGate => "Quality Gate",
            Self::CloseTicket => "Close Ticket",
        }
    }

    /// Prefixo de commit convencional associado à fase, se aplicável.
    pub fn commit_prefix(&self) -> Option<&'static str> {
        match self {
            Self::Ticket | Self::NewBranch => None,
            Self::SpecContract => Some("docs"),
            Self::RedTest => Some("test"),
            Self::GreenCode => Some("feat"),
            Self::Refactor => Some("refactor"),
            Self::MutationTest => Some("test"),
            Self::QualityGate => Some("fix"),
            Self::CloseTicket => Some("docs"),
        }
    }

    /// Valida se a fase corrente autoriza modificações em código executável de produção.
    pub fn allows_production_code(&self) -> bool {
        match self {
            Self::Ticket | Self::NewBranch | Self::SpecContract | Self::RedTest => false,
            Self::GreenCode
            | Self::Refactor
            | Self::MutationTest
            | Self::QualityGate
            | Self::CloseTicket => true,
        }
    }

    /// Retorna a próxima fase normativa válida no ciclo.
    pub fn next(&self) -> Option<Self> {
        match self {
            Self::Ticket => Some(Self::NewBranch),
            Self::NewBranch => Some(Self::SpecContract),
            Self::SpecContract => Some(Self::RedTest),
            Self::RedTest => Some(Self::GreenCode),
            Self::GreenCode => Some(Self::Refactor),
            Self::Refactor => Some(Self::MutationTest),
            Self::MutationTest => Some(Self::QualityGate),
            Self::QualityGate => Some(Self::CloseTicket),
            Self::CloseTicket => None,
        }
    }

    /// Converte um número inteiro (1..=9) para a fase correspondente.
    pub fn from_u8(n: u8) -> Result<Self, SessionError> {
        match n {
            1 => Ok(Self::Ticket),
            2 => Ok(Self::NewBranch),
            3 => Ok(Self::SpecContract),
            4 => Ok(Self::RedTest),
            5 => Ok(Self::GreenCode),
            6 => Ok(Self::Refactor),
            7 => Ok(Self::MutationTest),
            8 => Ok(Self::QualityGate),
            9 => Ok(Self::CloseTicket),
            _ => Err(SessionError::InvalidTransition { from: n, to: n }),
        }
    }

    /// Gera a diretiva normativa a ser injetada no prompt do Agente Externo para a fase ativa.
    pub fn agent_directive(&self, branch: &str) -> String {
        let prohibition = "PROIBIÇÃO EXPRESSA: Não execute comandos git ou cccp (commits, tags, merge, branches e fechamento de tickets são de responsabilidade exclusiva da PULT).";
        match self {
            Self::Ticket => format!(
                "★ [DIRETIVA GOST - FASE 1: TICKET/PLANO]\n\
                 Objetivo: Discutir, formular e planejar o escopo e arquitetura com a usuária.\n\
                 VETO: Não crie nem modifique nenhum arquivo de código ou de teste nesta fase.\n\
                 {}",
                prohibition
            ),
            Self::NewBranch => format!(
                "★ [DIRETIVA GOST - FASE 2: NEW BRANCH]\n\
                 Objetivo: Definição da branch de trabalho. Aguarde a PULT criar ou alternar a branch.\n\
                 {}",
                prohibition
            ),
            Self::SpecContract => format!(
                "★ [DIRETIVA GOST - FASE 3: SPEC & CONTRATO]\n\
                 Você tem autorização expressa para criar ou modificar APENAS arquivos de especificação em docs/ ou tipos/contratos de domínio na branch '{}'.\n\
                 VETO: É expressamente proibido criar ou modificar código de produção em src/.\n\
                 {}",
                branch, prohibition
            ),
            Self::RedTest => format!(
                "★ [DIRETIVA GOST - FASE 4: RED TEST]\n\
                 Você tem autorização expressa para criar ou modificar APENAS arquivos de testes automatizados na branch '{}', comprovando a falha inicial (Red).\n\
                 VETO: Não implemente código de produção nesta fase.\n\
                 {}",
                branch, prohibition
            ),
            Self::GreenCode => format!(
                "★ [DIRETIVA GOST - FASE 5: GREEN CODE]\n\
                 Você tem autorização expressa para implementar a solução mínima em código de produção na branch '{}' para fazer os testes passarem (Green).\n\
                 {}",
                branch, prohibition
            ),
            Self::Refactor => format!(
                "★ [DIRETIVA GOST - FASE 6: REFACTOR]\n\
                 Você tem autorização expressa para refatorar e polir o código na branch '{}', mantendo todos os testes verdes e eliminando avisos de linter.\n\
                 {}",
                branch, prohibition
            ),
            Self::MutationTest => format!(
                "★ [DIRETIVA GOST - FASE 7: MUTATION TEST]\n\
                 Você tem autorização expressa para adicionar testes de mutação e casos de borda na branch '{}' para eliminar mutantes e elevar a cobertura.\n\
                 {}",
                branch, prohibition
            ),
            Self::QualityGate => format!(
                "★ [DIRETIVA GOST - FASE 8: QUALITY GATE]\n\
                 Você tem autorização expressa para ajustes pontuais de formatação e linters na branch '{}' para aprovação no quality gate.\n\
                 {}",
                branch, prohibition
            ),
            Self::CloseTicket => format!(
                "★ [DIRETIVA GOST - FASE 9: CLOSE TICKET]\n\
                 Você tem autorização expressa para atualizar a crônica final e documentação (ex: CHANGELOG.md) na branch '{}'.\n\
                 {}",
                branch, prohibition
            ),
        }
    }
}
