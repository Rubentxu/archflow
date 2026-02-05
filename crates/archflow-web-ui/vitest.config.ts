import { defineConfig } from 'vitest/config';

export default defineConfig({
  test: {
    globals: true,
    environment: 'node',
    include: ['src/test/**/*.test.ts'],
    deps: {
      inline: ['archflow_web'],
    },
    server: {
      deps: {
        // Configurar directorios para archivos estáticos
        inline: [
          /archflow_web_bg\.wasm$/,
        ],
      },
    },
    // Usar el servidor de vite para servir archivos estáticos
    vite: {
      server: {
        fs: {
          // Permitir servir archivos desde estos directorios
          allow: ['..'],
        },
      },
    },
  },
});
