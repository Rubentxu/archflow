# EPIC-FASE-07: Migration

**Versión:** 1.0.0  
**Fase:** 7/8  
**Duración:** Semana 9  
**Referencia:** `MIGRACION_RECORDS_V2_COMPLETA.md` - Apéndice E, Criterios de Eliminación

---

## 📋 Descripción General

**ENFOQUE: MIGRACIÓN COMPLETA - SIN LEGACY**

Script automatizado para migrar código existente y verificar que ningún código legacy permanece en el sistema.

### Objetivos Principales
- `CodeMigrator` con reglas regex inteligentes
- `TestGenerator` para tests automatizados basados en el nuevo código
- `MigrationVerifier` para validación integral
- Verificar que ningún código legacy permanece
- Generar reporte de migración detallado

---

## 🔬 Investigación Perplexity Requerida

Antes de implementar, realizar investigación con Perplexity sobre:
- Rust codemod patterns 2024
- Automated refactoring toolchains
- Regex-based code transformation best practices
- AST-based vs regex-based migration strategies

**Prompt de investigación:**
```
Research Rust code migration patterns and tools 2024.
Focus on: 1) regex-based code transformation, 2) AST-based migration with syn crate,
3) automated test generation from migration, 4) validation strategies.
Include available tools and libraries with examples.
```

---

## 📦 Entregables (TODO DESDE CERO)

### Módulo 7.1: `src/migrator.rs` (NUEVO)

**TDD Test First:**
```rust
#[cfg(test)]
mod migrator_tests {
    use super::*;

    #[test]
    fn test_collect_rust_files() {
        let temp_dir = TempDir::new().unwrap();
        create_test_files(&temp_dir);

        let mut migrator = CodeMigrator::new(temp_dir.path(), temp_dir.path());
        let files = migrator.collect_rust_files(temp_dir.path()).unwrap();

        assert_eq!(files.len(), 3); // test1.rs, test2.rs, lib.rs
    }

    #[test]
    fn test_apply_simple_rule() {
        let rule = MigrationRule {
            name: "EntityId to RecordId",
            pattern: Regex::new(r"EntityId").unwrap(),
            replacement: "RecordId".to_string(),
        };

        let source = r#"let id = EntityId::new();"#;
        let result = rule.apply(source).unwrap();
        assert_eq!(result, r#"let id = RecordId::new();"#);
    }

    #[test]
    fn test_full_migration_report() {
        let temp_dir = TempDir::new().unwrap();
        create_test_files(&temp_dir);

        let mut migrator = CodeMigrator::new(temp_dir.path(), temp_dir.path());
        migrator.add_all_rules();
        let report = migrator.migrate_all().unwrap();

        assert_eq!(report.files_processed, 3);
        assert_eq!(report.rules_applied, 8);
        assert!(report.errors.is_empty());
    }
}
```

**Implementación:**
```rust
// Migration automatizada desde código legacy a Records
use regex::Regex;
use std::path::{Path, PathBuf};
use std::fs;
use serde::Serialize;

#[derive(Debug, Clone)]
pub struct CodeMigrator {
    source_root: PathBuf,
    target_root: PathBuf,
    rules: Vec<MigrationRule>,
}

#[derive(Debug, Clone)]
pub struct MigrationRule {
    name: &'static str,
    pattern: Regex,
    replacement: String,
}

impl MigrationRule {
    pub fn new(name: &'static str, pattern: &str, replacement: &str) -> Self {
        Self {
            name,
            pattern: Regex::new(pattern).unwrap(),
            replacement: replacement.to_string(),
        }
    }

    pub fn apply(&self, content: &str) -> Result<String, MigrationError> {
        let result = self.pattern.replace_all(content, &self.replacement);
        Ok(result.to_string())
    }
}

impl CodeMigrator {
    pub fn new(source_root: &Path, target_root: &Path) -> Self {
        Self {
            source_root: source_root.to_path_buf(),
            target_root: target_root.to_path_buf(),
            rules: Vec::new(),
        }
    }

    pub fn add_all_rules(&mut self) {
        // Reglas de EntityId → RecordId
        self.rules.push(MigrationRule::new(
            "EntityId to RecordId",
            r"\bEntityId\b",
            "RecordId",
        ));

        // Reglas de EntityStore → RecordStore
        self.rules.push(MigrationRule::new(
            "EntityStore to RecordStore",
            r"\bEntityStore\b",
            "RecordStore",
        ));

        // Reglas de event_sourcing → change_set
        self.rules.push(MigrationRule::new(
            "Event to ChangeSet",
            r"apply_event\((.*?),\s*(.*?)\)",
            "apply_delta($1, $2)",
        ));

        // Reglas de Primitive → Record
        self.rules.push(MigrationRule::new(
            "Primitive to Record",
            r"Primitive\s*\{([^}]*)\}",
            "RecordData {$1}",
        ));

        // ... más reglas según el documento de migración
    }

    pub fn collect_rust_files(&self, dir: &Path) -> Result<Vec<PathBuf>, MigrationError> {
        let mut files = Vec::new();

        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    self.collect_rust_files(&path)?;
                } else if let Some(ext) = path.extension() {
                    if ext == "rs" {
                        files.push(path);
                    }
                }
            }
        }

        Ok(files)
    }

    pub fn migrate_all(&mut self) -> Result<MigrationReport, MigrationError> {
        let mut files = self.collect_rust_files(&self.source_root)?;
        let mut report = MigrationReport::default();

        for file in &files {
            match self.migrate_file(file) {
                Ok(changes) => {
                    report.files_processed += 1;
                    report.changes_total += changes;
                }
                Err(e) => {
                    report.errors.push(e.to_string());
                }
            }
        }

        report.rules_applied = self.rules.len();
        Ok(report)
    }

    fn migrate_file(&self, source_path: &Path) -> Result<usize, MigrationError> {
        let content = fs::read_to_string(source_path)?;
        let relative_path = source_path.strip_prefix(&self.source_root)
            .unwrap_or(source_path);
        let target_path = self.target_root.join(relative_path);

        let mut migrated = content.clone();
        let mut changes = 0;

        for rule in &self.rules {
            let new_content = rule.apply(&migrated)?;
            if new_content != migrated {
                changes += 1;
            }
            migrated = new_content;
        }

        // Crear directorios padre si no existen
        if let Some(parent) = target_path.parent() {
            fs::create_dir_all(parent)?;
        }

        fs::write(&target_path, migrated)?;

        Ok(changes)
    }
}

#[derive(Debug, Default, Serialize)]
pub struct MigrationReport {
    pub files_processed: usize,
    pub rules_applied: usize,
    pub changes_total: usize,
    pub errors: Vec<String>,
    pub legacy_files_found: Vec<String>,
}
```

### Módulo 7.2: `src/verifier.rs` (NUEVO)

**TDD Test First:**
```rust
#[cfg(test)]
mod verifier_tests {
    use super::*;

    #[test]
    fn test_verify_compilation() {
        let verifier = MigrationVerifier;
        // Test con crate válido
        assert!(verifier.verify_compilation(Path::new("crates/archflow-records")).is_ok());
    }

    #[test]
    fn test_verify_no_legacy_references() {
        let verifier = MigrationVerifier;
        let temp_dir = TempDir::new().unwrap();

        // Crear archivo sin referencias legacy
        let clean_code = r#"
            use archflow_records::RecordStore;
            fn main() {
                let store = RecordStore::new();
            }
        "#;
        fs::write(temp_dir.path().join("lib.rs"), clean_code).unwrap();

        let result = verifier.verify_no_legacy_references(temp_dir.path());
        assert!(result.is_ok());

        // Crear archivo con referencia legacy
        let dirty_code = r#"
            use some_legacy::EntityId;
        "#;
        fs::write(temp_dir.path().join("dirty.rs"), dirty_code).unwrap();

        let result = verifier.verify_no_legacy_references(temp_dir.path());
        assert!(result.is_err());
    }

    #[test]
    fn test_find_legacy_patterns() {
        let verifier = MigrationVerifier;
        let code = r#"
            entity_id::EntityId
            entity_store::EntityStore
            apply_event
        "#;

        let patterns = verifier.find_legacy_patterns(code);
        assert_eq!(patterns.len(), 3);
    }
}
```

**Implementación:**
```rust
// Verificación de integridad post-migración
use std::path::Path;
use std::process::Command;

pub struct MigrationVerifier;

impl MigrationVerifier {
    /// Verificar que el código compila
    pub fn verify_compilation(&self, target_root: &Path) -> Result<(), VerificationError> {
        let output = Command::new("cargo")
            .args(&["check", "--manifest-path"])
            .arg(target_root.join("Cargo.toml"))
            .output()
            .map_err(|e| VerificationError::CommandFailed(e.to_string()))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(VerificationError::CompilationFailed(stderr.to_string()));
        }

        Ok(())
    }

    /// Verificar que no quedan referencias a código legacy
    pub fn verify_no_legacy_references(&self, target_root: &Path) -> Result<(), VerificationError> {
        let forbidden_patterns = [
            r"\bEntityId\b",
            r"\bEntityStore\b",
            r"Primitive\s*\{",
            r"apply_event",
            r"event_sourcing",
            r"EntityComponent",
        ];

        let mut found = Vec::new();

        // Buscar en todos los archivos .rs
        if let Ok(entries) = std::fs::read_dir(target_root) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() && path.extension().map(|e| e == "rs").unwrap_or(false) {
                    let content = std::fs::read_to_string(&path)?;
                    for pattern in &forbidden_patterns {
                        let regex = regex::Regex::new(pattern).unwrap();
                        if regex.is_match(&content) {
                            found.push(format!("{}: contains '{}'", path.display(), pattern));
                        }
                    }
                }
            }
        }

        if !found.is_empty() {
            return Err(VerificationError::LegacyReferencesFound(found));
        }

        Ok(())
    }

    /// Encontrar patrones legacy en código
    pub fn find_legacy_patterns(&self, code: &str) -> Vec<&str> {
        let patterns = [
            "EntityId",
            "EntityStore",
            "Primitive",
            "apply_event",
        ];

        patterns.iter()
            .filter(|&p| code.contains(p))
            .copied()
            .collect()
    }

    /// Verificar tests
    pub fn verify_tests(&self, target_root: &Path) -> Result<(), VerificationError> {
        let output = Command::new("cargo")
            .args(&["test", "--no-fail-fast"])
            .current_dir(target_root)
            .output()
            .map_err(|e| VerificationError::CommandFailed(e.to_string()))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(VerificationError::TestsFailed(stderr.to_string()));
        }

        Ok(())
    }
}

#[derive(Debug)]
pub enum VerificationError {
    CommandFailed(String),
    CompilationFailed(String),
    TestsFailed(String),
    LegacyReferencesFound(Vec<String>),
}
```

### Módulo 7.3: `src/main.rs` (CLI de migración)

```rust
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(StructOpt)]
struct Args {
    #[structopt(short, long)]
    source: PathBuf,
    #[structopt(short, long)]
    target: PathBuf,
    #[structopt(short, long)]
    verify: bool,
    #[structopt(short, long)]
    verbose: bool,
}

fn main() {
    let args = Args::from_args();

    println!("🔧 Iniciando migración...");
    println!("📂 Source: {}", args.source.display());
    println!("📂 Target: {}", args.target.display());

    let mut migrator = CodeMigrator::new(&args.source, &args.target);
    migrator.add_all_rules();

    match migrator.migrate_all() {
        Ok(report) => {
            println!("\n✅ Migración completada");
            println!("   Archivos procesados: {}", report.files_processed);
            println!("   Reglas aplicadas: {}", report.rules_applied);
            println!("   Cambios totales: {}", report.changes_total);

            if args.verify {
                let verifier = MigrationVerifier;
                if let Err(e) = verifier.verify_compilation(&args.target) {
                    eprintln!("❌ Error de compilación: {:?}", e);
                    std::process::exit(1);
                }
                println!("✅ Compilación exitosa");

                if let Err(e) = verifier.verify_no_legacy_references(&args.target) {
                    eprintln!("❌ Referencias legacy encontradas: {:?}", e);
                    std::process::exit(1);
                }
                println!("✅ Sin referencias legacy");
            }
        }
        Err(e) => {
            eprintln!("❌ Error en migración: {:?}", e);
            std::process::exit(1);
        }
    }
}
```

### Módulo 7.4: `ELIMINAR_TODO_LEGACY.sh` (Script final)

```bash
#!/bin/bash
# ELIMINAR_TODO_LEGACY.sh - Script final de eliminación de código legacy
# Este script elimina TODO el código legacy después de la migración

set -e

echo "=============================================="
echo "🗑️  ELIMINACIÓN FINAL DE CÓDIGO LEGACY"
echo "=============================================="
echo ""
echo "⚠️  ADVERTENCIA: Esta operación es IRREVERSIBLE"
echo "   Asegúrate de haber completado la migración primero"
echo ""

read -p "¿Continuar con la eliminación? (escribe 'SI' para confirmar): " confirm

if [ "$confirm" != "SI" ]; then
    echo "❌ Operación cancelada"
    exit 1
fi

echo ""
echo "🗑️  Eliminando código legacy..."

# ==============================================
# FASE 1: Records Foundation
# ==============================================
echo "   📦 Fase 1: Records Foundation..."
rm -f crates/archflow-core/src/entity_id.rs
rm -rf crates/archflow-core/src/event_sourcing/

# ==============================================
# FASE 2: Collaboration
# ==============================================
echo "   📦 Fase 2: Collaboration..."
rm -f crates/archflow-core/src/selection.rs
rm -f crates/archflow-core/src/connectivity.rs

# ==============================================
# FASE 3: Spatial
# ==============================================
echo "   📦 Fase 3: Spatial..."
rm -rf crates/archflow-geometry/

# ==============================================
# FASE 4: ECS Hybrid
# ==============================================
echo "   📦 Fase 4: ECS Hybrid..."
rm -rf crates/archflow-ecs/
rm -f crates/archflow-core/src/transform.rs

# ==============================================
# FASE 5: Renderer
# ==============================================
echo "   📦 Fase 5: Renderer..."
rm -rf crates/archflow-renderer/
rm -rf crates/archflow-renderer-canvas/
rm -rf crates/archflow-renderer-rough/

# ==============================================
# FASE 6: WASM
# ==============================================
echo "   📦 Fase 6: WASM..."
rm -rf crates/archflow-wasm/

# ==============================================
# VERIFICACIÓN FINAL
# ==============================================
echo ""
echo "🔍 Verificando eliminación..."

ERRORS=0

# Verificar que no existen los directorios
for dir in \
    crates/archflow-ecs \
    crates/archflow-geometry \
    crates/archflow-renderer \
    crates/archflow-renderer-canvas \
    crates/archflow-renderer-rough \
    crates/archflow-wasm; do
    if [ -e "$dir" ]; then
        echo "   ❌ Todavía existe: $dir"
        ERRORS=$((ERRORS + 1))
    else
        echo "   ✅ Eliminado: $dir"
    fi
done

# Verificar que no existen los archivos
for file in \
    crates/archflow-core/src/entity_id.rs \
    crates/archflow-core/src/transform.rs \
    crates/archflow-core/src/selection.rs \
    crates/archflow-core/src/connectivity.rs; do
    if [ -e "$file" ]; then
        echo "   ❌ Todavía existe: $file"
        ERRORS=$((ERRORS + 1))
    else
        echo "   ✅ Eliminado: $file"
    fi
done

# Verificar event_sourcing eliminado
if [ -d "crates/archflow-core/src/event_sourcing" ]; then
    echo "   ❌ Todavía existe: crates/archflow-core/src/event_sourcing"
    ERRORS=$((ERRORS + 1))
else
    echo "   ✅ Eliminado: crates/archflow-core/src/event_sourcing"
fi

echo ""
echo "=============================================="
if [ $ERRORS -eq 0 ]; then
    echo "✅ TODO EL CÓDIGO LEGACY HA SIDO ELIMINADO"
    echo "=============================================="
    echo ""
    echo "📊 Resumen de eliminación:"
    echo "   - entity_id.rs: ELIMINADO"
    echo "   - event_sourcing/: ELIMINADO"
    echo "   - selection.rs: ELIMINADO"
    echo "   - connectivity.rs: ELIMINADO"
    echo "   - archflow-geometry/: ELIMINADO"
    echo "   - archflow-ecs/: ELIMINADO"
    echo "   - transform.rs: ELIMINADO"
    echo "   - archflow-renderer/*: ELIMINADO"
    echo "   - archflow-wasm/: ELIMINADO"
    echo ""
    echo "🎉 La migración está completa. El código es 100% Records-based."
else
    echo "❌ ERRORS ENCONTRADOS: $ERRORS"
    echo "=============================================="
    echo "Revisa los errores anteriores antes de continuar"
    exit 1
fi
```

---

## 🎯 Criterios de Aceptación

| Criterio | Target | Método |
|----------|--------|--------|
| Zero legacy files | 0 archivos legacy remaining | Script de verificación |
| Compilation | 100% sin errores | `cargo check` |
| Tests | 100% passing | `cargo test` |
| Migration report | Generado y validado | Revisión manual |

---

## 📊 Referencias al Documento de Migración

| Sección | Contenido | Referencia |
|---------|-----------|------------|
| E.1 | Script de migración | L3500-3600 |
| E.2 | Verificación final | L3600-3650 |
| 7.1 | CodeMigrator | E.1 |
| 7.2 | MigrationVerifier | E.2 |

---

**Documento de Época: EPIC-FASE-07-Migration.md**  
**Versión:** 1.0.0  
**Creado:** 2026-01-26
