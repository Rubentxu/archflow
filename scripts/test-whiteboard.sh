#!/usr/bin/env bash
set -e

# ============================================================================
# ArchFlow Whiteboard Test Automation Script
# ============================================================================
# Este script automatiza las pruebas del whiteboard de ArchFlow:
# - Levanta el servidor de desarrollo
# - Espera a que esté listo
# - Ejecuta pruebas automatizadas del navegador
# - Captura logs y resultados
# ============================================================================

PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
WEB_UI_DIR="$PROJECT_ROOT/crates/archflow-web-ui"
LOG_DIR="$PROJECT_ROOT/test-logs"
TIMESTAMP=$(date +%Y%m%d-%H%M%S)
LOG_FILE="$LOG_DIR/whiteboard-test-$TIMESTAMP.log"
DEV_SERVER_PORT=5173
DEV_SERVER_URL="http://localhost:$DEV_SERVER_PORT"

# Colores para output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# ============================================================================
# Funciones auxiliares
# ============================================================================

log_info() {
    echo -e "${BLUE}[INFO]${NC} $1" | tee -a "$LOG_FILE"
}

log_success() {
    echo -e "${GREEN}[✓]${NC} $1" | tee -a "$LOG_FILE"
}

log_error() {
    echo -e "${RED}[✗]${NC} $1" | tee -a "$LOG_FILE"
}

log_warning() {
    echo -e "${YELLOW}[⚠]${NC} $1" | tee -a "$LOG_FILE"
}

cleanup() {
    log_info "Limpiando procesos..."
    if [ ! -z "$DEV_SERVER_PID" ]; then
        kill $DEV_SERVER_PID 2>/dev/null || true
        log_info "Servidor de desarrollo detenido (PID: $DEV_SERVER_PID)"
    fi
}

trap cleanup EXIT INT TERM

# ============================================================================
# Preparación
# ============================================================================

log_info "==================================================================="
log_info "ArchFlow Whiteboard - Pruebas Automatizadas"
log_info "==================================================================="
log_info "Timestamp: $TIMESTAMP"
log_info "Project Root: $PROJECT_ROOT"
log_info "Logs: $LOG_FILE"
log_info ""

# Crear directorio de logs
mkdir -p "$LOG_DIR"

# Verificar que WASM esté compilado
log_info "Verificando WASM compilado..."
if [ ! -d "$PROJECT_ROOT/crates/archflow-wasm-bridge/pkg" ]; then
    log_warning "WASM no encontrado, compilando..."
    cd "$PROJECT_ROOT"
    just build-wasm >> "$LOG_FILE" 2>&1
    log_success "WASM compilado"
else
    log_success "WASM encontrado"
fi

# ============================================================================
# Levantar servidor de desarrollo
# ============================================================================

log_info "Iniciando servidor de desarrollo..."
cd "$WEB_UI_DIR"

# Matar cualquier proceso anterior en el puerto
lsof -ti:$DEV_SERVER_PORT | xargs kill -9 2>/dev/null || true
sleep 1

# Iniciar servidor en background
npm run dev > "$LOG_DIR/dev-server-$TIMESTAMP.log" 2>&1 &
DEV_SERVER_PID=$!

log_info "Servidor iniciado (PID: $DEV_SERVER_PID)"
log_info "Esperando a que el servidor esté listo..."

# Esperar a que el servidor responda
MAX_WAIT=30
WAITED=0
while [ $WAITED -lt $MAX_WAIT ]; do
    if curl -s "$DEV_SERVER_URL" > /dev/null 2>&1; then
        log_success "Servidor listo en $DEV_SERVER_URL"
        break
    fi
    sleep 1
    WAITED=$((WAITED + 1))
    echo -n "."
done
echo ""

if [ $WAITED -ge $MAX_WAIT ]; then
    log_error "Timeout esperando al servidor"
    exit 1
fi

sleep 2  # Espera adicional para que WASM se inicialice

# ============================================================================
# Ejecutar pruebas
# ============================================================================

log_info ""
log_info "==================================================================="
log_info "EJECUTANDO PRUEBAS DEL WHITEBOARD"
log_info "==================================================================="
log_info ""

# Crear archivo de pruebas Node.js inline
TEST_SCRIPT="$LOG_DIR/test-runner-$TIMESTAMP.js"

cat > "$TEST_SCRIPT" << 'EOTEST'
// Test runner para ArchFlow Whiteboard
const puppeteer = require('puppeteer');

const DEV_SERVER_URL = process.env.DEV_SERVER_URL || 'http://localhost:5173';
const HEADLESS = process.env.HEADLESS !== 'false';

(async () => {
  console.log('🚀 Iniciando pruebas del whiteboard...\n');

  const browser = await puppeteer.launch({
    headless: HEADLESS ? 'new' : false,
    args: ['--no-sandbox', '--disable-setuid-sandbox']
  });

  const page = await browser.newPage();

  // Capturar logs de consola
  const logs = [];
  page.on('console', msg => {
    const text = msg.text();
    logs.push(text);
    console.log(`[BROWSER] ${text}`);
  });

  // Capturar errores
  const errors = [];
  page.on('pageerror', error => {
    errors.push(error.toString());
    console.error(`[ERROR] ${error}`);
  });

  try {
    // 1. Navegar a la aplicación
    console.log('📍 Navegando a', DEV_SERVER_URL);
    await page.goto(DEV_SERVER_URL, { waitUntil: 'networkidle0', timeout: 30000 });

    // 2. Esperar a que WASM se inicialice
    console.log('⏳ Esperando inicialización de WASM...');
    await page.waitForFunction(() => window.wasmInitialized === true, { timeout: 10000 });
    console.log('✅ WASM inicializado\n');

    // 3. Encontrar el canvas
    console.log('🎨 Buscando canvas del whiteboard...');
    const canvas = await page.$('canvas');
    if (!canvas) {
      throw new Error('Canvas no encontrado');
    }
    console.log('✅ Canvas encontrado\n');

    // 4. Seleccionar herramienta Rectangle
    console.log('🔧 Seleccionando herramienta Rectangle...');
    await page.evaluate(() => {
      const bridge = window.bridge;
      if (!bridge) throw new Error('Bridge no disponible');
      bridge.set_tool('Rectangle');
      console.log('🎯 Herramienta Rectangle seleccionada');
    });
    await page.waitForTimeout(500);

    // 5. Configurar colores de prueba
    console.log('🎨 Configurando colores...');
    await page.evaluate(() => {
      const bridge = window.bridge;
      // Color de relleno: azul (0x0000FF)
      bridge.set_active_color(0x0000FF);
      // Color de trazo: rojo (0xFF0000)
      bridge.set_active_stroke_color(0xFF0000);
      // Ancho de trazo: 3px
      bridge.set_active_stroke_width(3.0);
      console.log('✅ Colores configurados: fill=0x0000FF, stroke=0xFF0000, width=3.0');
    });
    await page.waitForTimeout(500);

    // 6. Obtener estado inicial
    const initialState = await page.evaluate(() => {
      const bridge = window.bridge;
      return {
        entityCount: bridge.get_entity_count(),
        historyLength: bridge.get_history_length()
      };
    });
    console.log(`📊 Estado inicial: ${initialState.entityCount} entidades, ${initialState.historyLength} en historial\n`);

    // 7. Dibujar un rectángulo
    console.log('✏️  Dibujando rectángulo en el canvas...');
    const box = await canvas.boundingBox();

    const startX = box.x + box.width / 2 - 50;
    const startY = box.y + box.height / 2 - 50;
    const endX = startX + 100;
    const endY = startY + 100;

    console.log(`   Inicio: (${Math.round(startX)}, ${Math.round(startY)})`);
    console.log(`   Fin: (${Math.round(endX)}, ${Math.round(endY)})`);

    await page.mouse.move(startX, startY);
    await page.mouse.down();
    await page.waitForTimeout(100);

    await page.mouse.move(endX, endY);
    await page.waitForTimeout(100);

    await page.mouse.up();
    await page.waitForTimeout(1000); // Esperar a que se procese

    console.log('✅ Rectángulo dibujado\n');

    // 8. Verificar resultado
    console.log('🔍 Verificando resultado...');
    const finalState = await page.evaluate(() => {
      const bridge = window.bridge;
      return {
        entityCount: bridge.get_entity_count(),
        historyLength: bridge.get_history_length()
      };
    });

    console.log(`📊 Estado final: ${finalState.entityCount} entidades, ${finalState.historyLength} en historial`);

    // 9. Obtener detalles de la última entidad creada
    if (finalState.entityCount > initialState.entityCount) {
      console.log('\n✅ Nueva entidad creada!');

      const entityDetails = await page.evaluate(() => {
        const bridge = window.bridge;
        const count = bridge.get_entity_count();
        if (count > 0) {
          // Obtener la última entidad
          return bridge.get_shapes();
        }
        return null;
      });

      if (entityDetails) {
        console.log('📋 Detalles de entidades:');
        console.log(JSON.stringify(entityDetails, null, 2));
      }
    } else {
      console.log('\n⚠️  No se creó ninguna entidad nueva');
    }

    // 10. Capturar screenshot
    console.log('\n📸 Capturando screenshot...');
    await page.screenshot({
      path: process.env.SCREENSHOT_PATH || 'whiteboard-test-screenshot.png',
      fullPage: true
    });
    console.log('✅ Screenshot guardado\n');

    // 11. Resumen
    console.log('='.repeat(70));
    console.log('RESUMEN DE PRUEBAS');
    console.log('='.repeat(70));
    console.log(`Entidades creadas: ${finalState.entityCount - initialState.entityCount}`);
    console.log(`Historial actualizado: ${finalState.historyLength > initialState.historyLength ? 'Sí' : 'No'}`);
    console.log(`Logs capturados: ${logs.length}`);
    console.log(`Errores encontrados: ${errors.length}`);

    if (errors.length > 0) {
      console.log('\n❌ ERRORES:');
      errors.forEach((err, i) => console.log(`  ${i + 1}. ${err}`));
    }

    // Buscar logs relevantes
    const creationLogs = logs.filter(l =>
      l.includes('Created') ||
      l.includes('shape') ||
      l.includes('entity') ||
      l.includes('🎨')
    );

    if (creationLogs.length > 0) {
      console.log('\n📝 LOGS DE CREACIÓN:');
      creationLogs.forEach(log => console.log(`  ${log}`));
    }

    console.log('='.repeat(70));

    // Resultado final
    const success = finalState.entityCount > initialState.entityCount && errors.length === 0;

    if (success) {
      console.log('\n✅ PRUEBAS EXITOSAS');
      process.exit(0);
    } else {
      console.log('\n❌ PRUEBAS FALLIDAS');
      process.exit(1);
    }

  } catch (error) {
    console.error('\n❌ Error en las pruebas:', error);
    await page.screenshot({ path: 'error-screenshot.png' });
    process.exit(1);
  } finally {
    await browser.close();
  }
})();
EOTEST

# Verificar si puppeteer está instalado
if ! npm list puppeteer > /dev/null 2>&1; then
    log_info "Instalando puppeteer..."
    cd "$PROJECT_ROOT"
    npm install --save-dev puppeteer >> "$LOG_FILE" 2>&1
    log_success "Puppeteer instalado"
fi

# Ejecutar pruebas
log_info "Ejecutando test runner..."
cd "$PROJECT_ROOT"

export DEV_SERVER_URL="$DEV_SERVER_URL"
export HEADLESS="${HEADLESS:-true}"
export SCREENSHOT_PATH="$LOG_DIR/screenshot-$TIMESTAMP.png"

if node "$TEST_SCRIPT" 2>&1 | tee -a "$LOG_FILE"; then
    log_success ""
    log_success "==================================================================="
    log_success "PRUEBAS COMPLETADAS EXITOSAMENTE"
    log_success "==================================================================="
    log_success "Logs: $LOG_FILE"
    log_success "Screenshot: $SCREENSHOT_PATH"
    exit 0
else
    log_error ""
    log_error "==================================================================="
    log_error "PRUEBAS FALLIDAS"
    log_error "==================================================================="
    log_error "Revisa los logs en: $LOG_FILE"
    exit 1
fi
