use std::fs::{create_dir_all, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::sync::Mutex;

/// Gravador thread-safe de trilha de auditoria e telemetria de interações em disco.
#[derive(Debug)]
pub struct AuditLogger {
    path: PathBuf,
    lock: Mutex<()>,
}

impl AuditLogger {
    /// Inicializa o gestor de log de auditoria garantindo que o diretório de destino exista.
    pub fn new(path: impl AsRef<Path>) -> io::Result<Self> {
        let p = path.as_ref().to_path_buf();
        if let Some(parent) = p.parent() {
            create_dir_all(parent)?;
        }
        // Testa criação/abertura do arquivo em modo append
        OpenOptions::new().create(true).append(true).open(&p)?;
        Ok(Self {
            path: p,
            lock: Mutex::new(()),
        })
    }

    /// Retorna o caminho absoluto ou relativo do arquivo de log configurado.
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Grava uma entrada de auditoria com timestamp ISO, categoria e mensagem.
    pub fn log(&self, category: &str, message: &str) {
        let _guard = self.lock.lock();
        if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(&self.path) {
            let now_sec = crate::domain::session::current_epoch_seconds();
            let timestamp = crate::domain::session::epoch_to_iso(now_sec);
            let _ = writeln!(file, "[{}] [{}] {}", timestamp, category, message);
        }
    }
}
