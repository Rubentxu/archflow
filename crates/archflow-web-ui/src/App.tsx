import { useState, useCallback } from "react";
import Header from "./components/Header";
import Toolbar from "./components/Toolbar";
import Canvas from "./components/Canvas";
import Sidebar from "./components/Sidebar";
import PropertiesPanel from "./components/PropertiesPanel";
import { useUIStore } from "./store/useUIStore";

function App() {
  const [darkMode, setDarkMode] = useState(false);
  const [activeTool, setActiveTool] = useState("select");
  const { isSidebarOpen, isPropertiesPanelOpen } = useUIStore();

  const toggleDarkMode = useCallback(() => {
    setDarkMode(!darkMode);
    document.documentElement.classList.toggle("dark");
  }, [darkMode]);

  return (
    <div
      className={`
        bg-background-light dark:bg-background-dark
        text-[#0d181b] dark:text-[#e0e0e0]
        font-display h-screen flex flex-col overflow-hidden
        ${darkMode ? "dark" : ""}
      `}
    >
      <Header darkMode={darkMode} onToggleDarkMode={toggleDarkMode} />

      <div className="flex flex-1 overflow-hidden">
        {isSidebarOpen && <Sidebar />}

        <main className="flex-1 relative overflow-hidden dot_grid">
          <Toolbar activeTool={activeTool} onToolChange={setActiveTool} />
          <Canvas />
        </main>

        {isPropertiesPanelOpen && (
          <PropertiesPanel selectedEntity={null} onEntityUpdate={() => {}} />
        )}
      </div>
    </div>
  );
}

export default App;
