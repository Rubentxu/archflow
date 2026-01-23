#!/bin/bash

# Script simplificado para analizar librerías con repomix

set -e

OUTPUT_DIR="./repo-analysis"
REPOS_DIR="./temp-repos"
mkdir -p "$OUTPUT_DIR"

echo "🔍 Analyzing diagramming libraries for Rust port..."
echo ""

# 1. tldraw - EL MÁS IMPORTANTE
echo "📦 [1/3] Analyzing tldraw (RECORD SYSTEM - PRIMARY TARGET)..."
npx repomix "$REPOS_DIR/tldraw" \
  --output "$OUTPUT_DIR/tldraw-core.xml" \
  --style xml \
  --include "packages/store/**/*.ts" \
  --include "packages/record/**/*.ts" \
  --include "packages/utils/src/**/*.ts" \
  --compress

echo "   ✅ tldraw analyzed"
echo ""

# 2. React Flow - Para conexiones
echo "📦 [2/3] Analyzing React Flow (for CONNECTION model)..."
npx repomix "$REPOS_DIR/xyflow" \
  --output "$OUTPUT_DIR/reactflow-core.xml" \
  --style xml \
  --include "packages/reactflow/src/**/*.ts" \
  --include "packages/system/src/**/*.ts" \
  --compress

echo "   ✅ React Flow analyzed"
echo ""

# 3. Excalidraw - Menos prioritario
echo "📦 [3/3] Analyzing Excalidraw (for element types)..."
npx repomix "$REPOS_DIR/excalidraw" \
  --output "$OUTPUT_DIR/excalidraw-core.xml" \
  --style xml \
  --include "packages/excalidraw/element/**/*.ts" \
  --include "packages/excalidraw/scene/**/*.ts" \
  --compress

echo "   ✅ Excalidraw analyzed"
echo ""

echo "✨ Analysis complete!"
echo ""
echo "📊 Results:"
for file in "$OUTPUT_DIR"/*.xml; do
    if [ -f "$file" ]; then
        size=$(wc -c < "$file")
        echo "   $(basename "$file"): $size bytes"
    fi
done
echo ""
echo "📁 All files in: $OUTPUT_DIR"
