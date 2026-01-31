import { useState } from "react";
import Header from "./components/Header";
import Toolbar from "./components/Toolbar";
import Canvas from "./components/Canvas";
import Sidebar from "./components/Sidebar";
import PropertiesPanel from "./components/PropertiesPanel";

function App() {
  const [darkMode, setDarkMode] = useState(false);
  const [selectedEntity, setSelectedEntity] = useState<number | null>(null);

  const toggleDarkMode = () => {
    setDarkMode(!darkMode);
    document.documentElement.classList.toggle("dark");
  };

  return (
    <div
      className={`bg-background-light dark:bg-background-dark text-[#0d181b] dark:text-[#e0e0e0] font-display h-screen flex flex-col overflow-hidden ${darkMode ? "dark" : ""}`}
    >
      <Header darkMode={darkMode} onToggleDarkMode={toggleDarkMode} />

      <div className="flex flex-1 overflow-hidden">
        <Sidebar />

        <main className="flex-1 relative overflow-hidden dot-grid">
          <Toolbar />
          <Canvas
            selectedEntity={selectedEntity}
            onSelectEntity={setSelectedEntity}
          />
        </main>

        <PropertiesPanel selectedEntity={selectedEntity} />
      </div>
    </div>
  );
}

export default App;
