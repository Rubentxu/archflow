#!/bin/bash
# ═══════════════════════════════════════════════════════════════════════════════
# ArchFlow WASM Type Sync Script
#
# Este script sincroniza los archivos generados por wasm-pack desde
# crates/archflow-web/pkg/ hacia crates/archflow-web-ui/src/wasm/
#
# La fuente de la verdad es el código Rust. Este script mantiene el frontend
# sincronizado automáticamente.
#
# Uso: ./scripts/sync-wasm-types.sh
#      just sync-wasm-types
# ═══════════════════════════════════════════════════════════════════════════════

set -e  # Exit on error
set -u  # Exit on undefined variable

# Colores para output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Función para print con color
print_status() {
    echo -e "${BLUE}[Sync]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[OK]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# ═══════════════════════════════════════════════════════════════════════════════
# CONFIGURACIÓN
# ═══════════════════════════════════════════════════════════════════════════════

# Directorios
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
WASM_PKG_DIR="$PROJECT_ROOT/crates/archflow-web/pkg"
FRONTEND_WASM_DIR="$PROJECT_ROOT/crates/archflow-web-ui/src/wasm"

# Archivos a sincronizar
TYPE_FILES=(
    "archflow_web.d.ts"
    "archflow_web_bg.d.ts"
    "archflow_web_bg.wasm.d.ts"
)

JS_FILES=(
    "archflow_web.js"
    "archflow_web_bg.js"
)

WASM_FILES=(
    "archflow_web_bg.wasm"
)

# ═══════════════════════════════════════════════════════════════════════════════
# VERIFICACIONES INICIALES
# ═══════════════════════════════════════════════════════════════════════════════

print_status "Iniciando sincronización de tipos WASM..."

# Verificar que existe el directorio source
if [ ! -d "$WASM_PKG_DIR" ]; then
    print_error "No existe el directorio WASM pkg: $WASM_PKG_DIR"
    print_status "Ejecuta 'just build-wasm' primero para generar el WASM"
    exit 1
fi

# Verificar que existe el directorio destino
if [ ! -d "$FRONTEND_WASM_DIR" ]; then
    print_warning "No existe el directorio destino: $FRONTEND_WASM_DIR"
    print_status "Creando directorio..."
    mkdir -p "$FRONTEND_WASM_DIR"
fi

# ═══════════════════════════════════════════════════════════════════════════════
# SINCRONIZACIÓN DE TIPOS (.d.ts)
# ═══════════════════════════════════════════════════════════════════════════════

print_status "Sincronizando archivos de tipos..."

types_synced=0
types_failed=0

for file in "${TYPE_FILES[@]}"; do
    source_file="$WASM_PKG_DIR/$file"
    dest_file="$FRONTEND_WASM_DIR/$file"

    if [ -f "$source_file" ]; then
        if [ -f "$dest_file" ]; then
            # Comparar si hay cambios
            if ! diff -q "$source_file" "$dest_file" > /dev/null 2>&1; then
                cp "$source_file" "$dest_file"
                print_success "Actualizado: $file"
                ((types_synced++))
            else
                print_status "Sin cambios: $file"
            fi
        else
            # Archivo nuevo
            cp "$source_file" "$dest_file"
            print_success "Nuevo archivo: $file"
            ((types_synced++))
        fi
    else
        print_warning "No encontrado: $file"
        ((types_failed++))
    fi
done

# ═══════════════════════════════════════════════════════════════════════════════
# SINCRONIZACIÓN DE JS BINDINGS
# ═══════════════════════════════════════════════════════════════════════════════

print_status "Sincronizando archivos JavaScript..."

js_synced=0
js_failed=0

for file in "${JS_FILES[@]}"; do
    source_file="$WASM_PKG_DIR/$file"
    dest_file="$FRONTEND_WASM_DIR/$file"

    if [ -f "$source_file" ]; then
        if [ -f "$dest_file" ]; then
            # Comparar si hay cambios
            if ! diff -q "$source_file" "$dest_file" > /dev/null 2>&1; then
                cp "$source_file" "$dest_file"
                print_success "Actualizado: $file"
                ((js_synced++))
            else
                print_status "Sin cambios: $file"
            fi
        else
            # Archivo nuevo
            cp "$source_file" "$dest_file"
            print_success "Nuevo archivo: $file"
            ((js_synced++))
        fi
    else
        print_warning "No encontrado: $file"
        ((js_failed++))
    fi
done

# ═══════════════════════════════════════════════════════════════════════════════
# SINCRONIZACIÓN DE WASM (solo si cambió)
# ═══════════════════════════════════════════════════════════════════════════════

print_status "Sincronizando archivo WASM..."

wasm_synced=0
wasm_failed=0

for file in "${WASM_FILES[@]}"; do
    source_file="$WASM_PKG_DIR/$file"
    dest_file="$FRONTEND_WASM_DIR/$file"

    if [ -f "$source_file" ]; then
        if [ -f "$dest_file" ]; then
            # Comparar por tamaño y fecha de modificación
            source_size=$(stat -c%s "$source_file" 2>/dev/null || stat -f%z "$source_file" 2>/dev/null)
            dest_size=$(stat -c%s "$dest_file" 2>/dev/null || stat -f%z "$dest_file" 2>/dev/null)

            if [ "$source_size" != "$dest_size" ]; then
                cp "$source_file" "$dest_file"
                print_success "Actualizado WASM: $file ($source_size bytes)"
                ((wasm_synced++))
            else
                print_status "Sin cambios: $file"
            fi
        else
            # Archivo nuevo
            cp "$source_file" "$dest_file"
            print_success "Nuevo archivo WASM: $file"
            ((wasm_synced++))
        fi
    else
        print_warning "No encontrado: $file"
        ((wasm_failed++))
    fi
done

# ═══════════════════════════════════════════════════════════════════════════════
# VERIFICACIÓN FINAL
# ═══════════════════════════════════════════════════════════════════════════════

echo ""
print_status "═══════════════════════════════════════════════════════════════════════"
print_status "Resumen de Sincronización"
print_status "═══════════════════════════════════════════════════════════════════════"
echo ""
echo "  Tipos TypeScript: $types_synced sincronizados, $types_failed fallidos"
echo "  JavaScript:      $js_synced sincronizados, $js_failed fallidos"
echo "  WASM:            $wasm_synced sincronizados, $wasm_failed fallidos"
echo ""

# Verificar que los archivos principales existen
if [ -f "$FRONTEND_WASM_DIR/archflow_web.d.ts" ] && \
   [ -f "$FRONTEND_WASM_DIR/archflow_web.js" ] && \
   [ -f "$FRONTEND_WASM_DIR/archflow_web_bg.wasm" ]; then
    print_success "Sincronización completada exitosamente!"
    echo ""
    print_status "Archivos sincronizados:"
    ls -lh "$FRONTEND_WASM_DIR"/*.d.ts 2>/dev/null || true
    ls -lh "$FRONTEND_WASM_DIR"/*.js 2>/dev/null | head -3 || true
    ls -lh "$FRONTEND_WASM_DIR"/*.wasm 2>/dev/null || true
    echo ""
    print_status "El frontend está listo para usar los tipos actualizados."
    exit 0
else
    print_error "Fallo en la sincronización - faltan archivos críticos"
    exit 1
fi
