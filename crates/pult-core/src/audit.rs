use std::fs::{create_dir_all, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::sync::{Mutex, RwLock};

/// Gravador thread-safe de trilha de auditoria e telemetria de interações em disco.
#[derive(Debug)]
pub struct AuditLogger {
    path: RwLock<PathBuf>,
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
            path: RwLock::new(p),
            lock: Mutex::new(()),
        })
    }

    /// Retorna uma cópia do caminho do arquivo de log ativo.
    pub fn file_path(&self) -> PathBuf {
        self.path.read().unwrap().clone()
    }

    /// Retorna uma cópia do caminho do arquivo de log configurado (compatibilidade).
    pub fn path(&self) -> PathBuf {
        self.file_path()
    }

    /// Atualiza o arquivo de destino do logger, garantindo que o novo diretório pai exista.
    pub fn retarget(&self, new_path: PathBuf) -> io::Result<()> {
        let _guard = self.lock.lock();
        if let Some(parent) = new_path.parent() {
            create_dir_all(parent)?;
        }
        OpenOptions::new()
            .create(true)
            .append(true)
            .open(&new_path)?;
        *self.path.write().unwrap() = new_path;
        Ok(())
    }

    /// Grava uma entrada de auditoria com timestamp ISO, categoria e mensagem.
    pub fn log(&self, category: &str, message: &str) {
        let _guard = self.lock.lock();
        let target_path = self.path.read().unwrap().clone();
        if let Ok(mut file) = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&target_path)
        {
            let now_sec = crate::domain::session::current_epoch_seconds();
            let timestamp = crate::domain::session::epoch_to_iso(now_sec);
            let _ = writeln!(file, "[{}] [{}] {}", timestamp, category, message);
        }
    }
}
