# ArchFlow Web - Guía de Build y Deployment

**Última actualización:** 2025-02-03  
**Estado:** Funcional (versión simple www/index.html)

---

## Resumen Ejecutivo

La aplicación ArchFlow tiene dos interfaces web:
1. **Simple (funciona):** `crates/archflow-web/www/index.html` - HTML puro con WASM
2. **React (en desarrollo):** `crates/archflow-web-ui/` - React + Vite + WASM (problemas con bundling)

---

## Arquitectura Actual

```
archflow-web/
├── src/           # Código Rust del engine
├── pkg/           # Salida de wasm-pack (--target web)
├── www/
│   └── index.html  # Interfaz simple HTML + JS (FUNCIONAL)
└── Cargo.toml

archflow-web-ui/
├── src/           # React + TypeScript
├── pkg/           # Archivos WASM copiados desde archflow-web/pkg/
├── dist/          # Salida de Vite build
├── vite.config.ts # Configuración Vite
└── package.json

archflow-web-server/
├── src/lib.rs     # Servidor Axum con headers COEP/COOP
└── bin/server.rs
```

---

## Comandos de Build y Deployment

### Desarrollo Rápido (Recomendado)

```bash
# 1. Construir WASM
cd crates/archflow-web
wasm-pack build --dev --target web

# 2. Servir con Vite en modo dev (con hot reload)
cd crates/archflow-web-ui
npm run dev

# En otra terminal, iniciar servidor Rust con COEP/COEP
cargo run -p archflow-web-server --bin server
# O: just server
```

### Producción (Build Completo)

```bash
# Usar justfile (automatización completa)
just build-web

# Iniciar servidor
just server

# O manualmente:
# 1. Build WASM
cd crates/archflow-web && wasm-pack build --dev --target web && cd ../..

# 2. Build React UI
cd crates/archflow-web-ui && npm run build && cd ../..

# 3. Copiar WASM a dist/
mkdir -p crates/archflow-web-ui/dist
cp crates/archflow-web/pkg/*.{js,wasm,d.ts} crates/archflow-web-ui/dist/

# 4. Iniciar servidor
cargo run -p archflow-web-server --bin server --dist crates/archflow-web-ui/dist --port 3000
```

---

## Problemas Conocidos y Soluciones

### Problema 1: BigInt Conversion Errors

**Síntoma:** `The number X cannot be converted to a BigInt because it is not an integer`

**Causa:** `performance.now()` devuelve float con decimales, `BigInt()` requiere enteros.

**Solución:** En `archflow-web/www/index.html`, línea 170:
```javascript
// Antes:
inputView.setBigUint64(offset, BigInt(timestamp), true);

// Después:
inputView.setBigUint64(offset, BigInt(Math.floor(timestamp * 1000)), true);
```

### Problema 2: TypeScript camelCase vs snake_case

**Síntoma:** Errores de TypeScript en React al importar funciones WASM

**Causa:** `wasm-bindgen` genera nombres `snake_case` (ej: `get_alive_entities`), TypeScript espera `camelCase`

**Solución:** Actualizar imports de React a usar `snake_case`:
- `pushInputEvent` → `push_input_event`
- `getAliveEntities` → `get_alive_entities`
- etc.

Archivos afectados:
- `src/hooks/useInput.ts`
- `src/hooks/useSelection.ts`
- `src/types/wasm.ts`

### Problema 3: Vite Production Build - WASM no se carga

**Síntoma:** "Initializing renderer..." se queda pegado, no errores en consola

**Causa:** Vite hace bundle del módulo WASM y cambia los nombres de archivo con hash, rompiendo las referencias internas.

**Estado:** EN INVESTIGACIÓN

**Solución propuesta:**
1. Usar `vite-plugin-wasm` + `vite-plugin-top-level-await` ✅ (ya instalado)
2. Mantener archivos WASM en `pkg/` fuera de `src/` ✅ (ya hecho)
3. Configurar `assetFileNames` para NO agregar hash a archivos `.wasm` y `.js` glue ⚠️ (pendiente)
4. Alternativa: Usar Vite dev mode en desarrollo

---

## Headers COEP/COOP para SharedArrayBuffer

El servidor `archflow-web-server` ya configura los headers necesarios:

```rust
// crates/archflow-web-server/src/lib.rs
headers.insert("Cross-Origin-Opener-Policy", "same-origin".parse().unwrap());
headers.insert("Cross-Origin-Embedder-Policy", "credentialless".parse().unwrap());  // Not require-corp
headers.insert("Cross-Origin-Resource-Policy", "cross-origin".parse().unwrap());
```

**Importante:** `credentialless` permite cargar recursos cross-origin (como WASM bundles) mientras mantiene aislamiento.

---

## Automatización con justfile

El `justfile` ya tiene recetas idempotentes:

```bash
just build-web    # Construye WASM + React, copia a dist/
just server       # Inicia servidor (mata servidor existente primero)
just dev          # build-web + server en uno
just kill-server # Detiene servidor
just status       # Verifica estado del entorno
just reset        # Limpia todo y reinstala dependencias
```

---

## Referencias

- [Using Rust WebAssembly in Vite + React](https://dev.to/jambochen/using-rust-webassembly-in-vite-react-a-modern-game-of-life-example-hde)
- [vite-plugin-wasm](https://www.npmjs.com/package/vite-plugin-wasm)
- [vite-plugin-top-level-await](https://www.npmjs.com/package/vite-plugin-top-level-await)
- [wasm-pack Issue #1106](https://github.com/rustwasm/wasm-pack/issues/1106)

---

## Próximos Pasos

1. **Inmediato:** Usar Vite dev mode para desarrollo con hot reload
2. **Corto plazo:** Configurar correctamente Vite para production build
3. **Largo plazo:** Considerar alternativas como usar `wasm-bindgen` con `--target bundler`