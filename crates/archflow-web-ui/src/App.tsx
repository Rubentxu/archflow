import { useState, useCallback } from "react";
import Header from "./components/Header";
import Toolbar from "./components/Toolbar";
import Canvas from "./components/Canvas";
import Sidebar from "./components/Sidebar";
import PropertiesPanel from "./components/PropertiesPanel";

function App() {
  const [darkMode, setDarkMode] = useState(false);
  const [selectedEntity, setSelectedEntity] = useState<number | null>(null);
  const [activeTool, setActiveTool] = useState("select");
  const [entityUpdateTrigger, setEntityUpdateTrigger] = useState(0);

  const toggleDarkMode = () => {
    setDarkMode(!darkMode);
    document.documentElement.classList.toggle("dark");
  };

  /**
   * Trigger entity update refresh
   * Called when properties are modified in the PropertiesPanel
   */
  const handleEntityUpdate = useCallback(() => {
    setEntityUpdateTrigger((prev) => prev + 1);
  }, []);

  /**
   * Handle entity selection from Canvas or Sidebar
   */
  const handleSelectEntity = useCallback((id: number | null) => {
    setSelectedEntity(id);
  }, []);

  return (
    <div
      className={`bg-background-light dark:bg-background-dark text-[#0d181b] dark:text-[#e0e0e0] font-display h-screen flex flex-col overflow-hidden ${darkMode ? "dark" : ""}`}
    >
      <Header darkMode={darkMode} onToggleDarkMode={toggleDarkMode} />

      <div className="flex flex-1 overflow-hidden">
        <Sidebar
          onEntitySelect={handleSelectEntity}
          selectedEntity={selectedEntity}
        />

        <main className="flex-1 relative overflow-hidden dot_grid">
          <Toolbar activeTool={activeTool} onToolChange={setActiveTool} />
          <Canvas
            key={entityUpdateTrigger}
            selectedEntity={selectedEntity}
            onSelectEntity={handleSelectEntity}
          />
        </main>

        <PropertiesPanel
          selectedEntity={selectedEntity}
          onEntityUpdate={handleEntityUpdate}
        />
      </div>
    </div>
  );
}

export default App;
