use crate::ast::error::AstError;
use crate::ast::extractor::AstExtractor;
use crate::ast::models::ExtractedAstFile;
use quote::ToTokens;
use std::fs;
use std::path::Path;
use syn::{Item, Visibility};
use walkdir::WalkDir;

/// Extrator estrutural de AST de alta performance para a linguagem Rust via `syn` e `quote`.
pub struct RustAstExtractor;

impl Default for RustAstExtractor {
    fn default() -> Self {
        Self::new()
    }
}

impl RustAstExtractor {
    pub fn new() -> Self {
        Self
    }

    fn is_pub(vis: &Visibility) -> bool {
        matches!(vis, Visibility::Public(_))
    }
}

impl AstExtractor for RustAstExtractor {
    fn extract_from_str(&self, file_path: &Path, content: &str) -> Result<String, AstError> {
        let file = syn::parse_file(content).map_err(|e| AstError::ParseError {
            file: file_path.to_string_lossy().to_string(),
            message: e.to_string(),
        })?;

        let mut output = String::new();

        for item in file.items {
            match item {
                Item::Struct(mut s) => {
                    if Self::is_pub(&s.vis) {
                        // Filtra campos privados mantendo apenas campos públicos
                        if let syn::Fields::Named(ref mut fields) = s.fields {
                            fields.named = fields
                                .named
                                .clone()
                                .into_pairs()
                                .filter(|pair| Self::is_pub(&pair.value().vis))
                                .collect();
                        }
                        output.push_str(&s.to_token_stream().to_string());
                        output.push_str("\n\n");
                    }
                }
                Item::Enum(e) => {
                    if Self::is_pub(&e.vis) {
                        output.push_str(&e.to_token_stream().to_string());
                        output.push_str("\n\n");
                    }
                }
                Item::Trait(mut t) => {
                    if Self::is_pub(&t.vis) {
                        // Remove corpos padrões de métodos do trait
                        for trait_item in &mut t.items {
                            if let syn::TraitItem::Fn(ref mut m) = trait_item {
                                m.default = None;
                                m.semi_token = Some(syn::token::Semi::default());
                            }
                        }
                        output.push_str(&t.to_token_stream().to_string());
                        output.push_str("\n\n");
                    }
                }
                Item::Type(t) => {
                    if Self::is_pub(&t.vis) {
                        output.push_str(&t.to_token_stream().to_string());
                        output.push_str("\n\n");
                    }
                }
                Item::Const(c) => {
                    if Self::is_pub(&c.vis) {
                        output.push_str(&c.to_token_stream().to_string());
                        output.push_str("\n\n");
                    }
                }
                Item::Fn(f) => {
                    if Self::is_pub(&f.vis) {
                        let sig = f.sig.to_token_stream().to_string();
                        output.push_str(&format!("pub {};\n\n", sig));
                    }
                }
                Item::Impl(i) => {
                    let is_trait_impl = i.trait_.is_some();
                    let mut signatures = Vec::new();

                    for impl_item in &i.items {
                        match impl_item {
                            syn::ImplItem::Fn(m) => {
                                if is_trait_impl || Self::is_pub(&m.vis) {
                                    let vis_str = if Self::is_pub(&m.vis) { "pub " } else { "" };
                                    let sig = m.sig.to_token_stream().to_string();
                                    signatures.push(format!("    {}{};", vis_str, sig));
                                }
                            }
                            syn::ImplItem::Type(t) if is_trait_impl || Self::is_pub(&t.vis) => {
                                signatures.push(format!("    {};", t.to_token_stream()));
                            }
                            syn::ImplItem::Const(c) if is_trait_impl || Self::is_pub(&c.vis) => {
                                signatures.push(format!("    {};", c.to_token_stream()));
                            }
                            _ => {}
                        }
                    }

                    if !signatures.is_empty() {
                        let self_ty = i.self_ty.to_token_stream().to_string();
                        let header = if let Some((_, ref trait_path, _)) = i.trait_ {
                            format!("impl {} for {}", trait_path.to_token_stream(), self_ty)
                        } else {
                            format!("impl {}", self_ty)
                        };

                        output.push_str(&header);
                        output.push_str(" {\n");
                        for sig in signatures {
                            output.push_str(&sig);
                            output.push('\n');
                        }
                        output.push_str("}\n\n");
                    }
                }
                _ => {}
            }
        }

        Ok(output.trim().to_string())
    }

    fn extract_from_file(&self, file_path: &Path) -> Result<ExtractedAstFile, AstError> {
        let content = fs::read_to_string(file_path).map_err(|e| AstError::Io {
            path: file_path.to_path_buf(),
            source: e,
        })?;

        let raw_byte_count = content.len();
        let extracted_signatures = self.extract_from_str(file_path, &content)?;
        let reduced_byte_count = extracted_signatures.len();
        let relative_path = file_path.to_string_lossy().to_string();

        Ok(ExtractedAstFile {
            file_path: file_path.to_path_buf(),
            relative_path,
            extracted_signatures,
            raw_byte_count,
            reduced_byte_count,
        })
    }

    fn extract_from_dir(
        &self,
        root: &Path,
        ignore_dirs: &[&str],
    ) -> Result<Vec<ExtractedAstFile>, AstError> {
        let mut results = Vec::new();

        for entry in WalkDir::new(root)
            .into_iter()
            .filter_entry(|e| {
                if e.depth() == 0 {
                    return true;
                }
                let name = e.file_name().to_string_lossy();
                if e.file_type().is_dir()
                    && (ignore_dirs.contains(&name.as_ref()) || name.starts_with('.'))
                {
                    return false;
                }
                true
            })
            .filter_map(|e| e.ok())
        {
            if entry.file_type().is_file() {
                if let Some(ext) = entry.path().extension().and_then(|x| x.to_str()) {
                    if ext == "rs" {
                        let rel_path = entry
                            .path()
                            .strip_prefix(root)
                            .unwrap_or(entry.path())
                            .to_string_lossy()
                            .to_string();

                        let mut file_ast = self.extract_from_file(entry.path())?;
                        file_ast.relative_path = rel_path;
                        results.push(file_ast);
                    }
                }
            }
        }

        results.sort_by(|a, b| a.relative_path.cmp(&b.relative_path));
        Ok(results)
    }
}
