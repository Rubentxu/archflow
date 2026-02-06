import { StrictMode } from 'react'
import { createRoot } from 'react-dom/client'
import './index.css'
import { ArchFlowWasmProvider } from './hooks/useArchFlowWasm.tsx'
import App from './App.tsx'

// Entry point
createRoot(document.getElementById('root')!).render(
  <StrictMode>
    <ArchFlowWasmProvider>
      <App />
    </ArchFlowWasmProvider>
  </StrictMode>,
)
