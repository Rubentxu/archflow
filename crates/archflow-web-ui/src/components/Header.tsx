/**
 * Header Component - Application Top Bar
 */

import {
  Rocket,
  Menu,
} from "lucide-react";
import { useUIStore } from "../store/useUIStore";
import { cn } from "../utils/cn";

interface HeaderProps {
  className?: string;
  projectName?: string;
  onSave?: () => void;
  onLoad?: () => void;
  onExport?: () => void;
  onSettings?: () => void;
}

export default function Header({
  className,
}: HeaderProps) {
  const { toggleSidebar } = useUIStore();

  return (
    <header
      className={cn(
        "h-14 flex items-center justify-between px-6 border-b border-border-light dark:border-border-dark",
        "bg-surface-light dark:bg-surface-dark shrink-0 z-30",
        className,
      )}
    >
      <div className="flex items-center gap-6">
        <div className="flex items-center gap-2 text-[#0d181b] dark:text-white">
          <button
            className="p-1.5 rounded-lg hover:bg-black/10 dark:hover:bg-white/10 transition-colors lg:hidden"
            onClick={toggleSidebar}
          >
            <Menu className="w-5 h-5 text-gray-500 dark:text-gray-400" />
          </button>
          <div className="size-6 text-primary flex items-center justify-center">
            {/* Using a simple icon replacement for the material symbol 'deployed_code' */}
            <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="currentColor" className="w-6 h-6">
              <path d="M12 2L2 7l10 5 10-5-10-5zm0 9l2.5-1.25L12 8.5l-2.5 1.25L12 11zm0 2.5l-5-2.5-5 2.5L12 22l10-8.5-5-2.5-5 2.5z" />
            </svg>
          </div>
          <h2 className="text-xl font-bold leading-tight tracking-[-0.015em]">ArchFlow</h2>
        </div>

        {/* Breadcrumbs */}
        <div className="h-6 w-px bg-border-light dark:bg-white/10 mx-2 hidden sm:block"></div>
        <div className="hidden sm:flex items-center gap-2 text-sm">
          <span className="text-slate-500 dark:text-slate-400 font-medium">Team Workspace</span>
          <span className="text-slate-300 dark:text-slate-600">/</span>
          <span className="text-slate-500 dark:text-slate-400 font-medium">Project Alpha</span>
          <span className="text-slate-300 dark:text-slate-600">/</span>
          <span className="text-[#0d181b] dark:text-white font-bold">Architecture V1</span>
        </div>
      </div>

      <div className="flex items-center gap-4">
        <div className="flex items-center gap-6 mr-4 hidden md:flex">
          <button className="text-slate-600 dark:text-slate-300 hover:text-primary transition-colors text-sm font-medium">File</button>
          <button className="text-slate-600 dark:text-slate-300 hover:text-primary transition-colors text-sm font-medium">Edit</button>
          <button className="text-slate-600 dark:text-slate-300 hover:text-primary transition-colors text-sm font-medium">View</button>
          <button className="text-slate-600 dark:text-slate-300 hover:text-primary transition-colors text-sm font-medium">Help</button>
        </div>

        {/* Avatar Group */}
        <div className="flex -space-x-2">
          <div className="relative inline-block">
            <div className="w-8 h-8 rounded-full border-2 border-white dark:border-surface-dark bg-cover bg-center bg-gray-300"
              style={{ backgroundImage: "url('https://lh3.googleusercontent.com/aida-public/AB6AXuCZz2zC4QuZ416Ac930ywOaXh7SmLUaNX3JKzmPAaRIuZFX_EjCl5hcyLUgkfeWKYeA-kBwUY4eThcEOTT7SwJrpmoHsg137GyosICeQ7DGDaggGQ_nFq6JM3eVErz0VLrM8jnoh0ClYogZR7mVSjR3--zc8in9CmarDO9Z058bihLGyXR3LDhsqeIMFLjk3lzs1LG4hXGXsD4FIzuCd88QbefYJ7QYgi7OToYOnw6576Xn_NDSSb8yMfFAp2GAz7zN95OezN1YIXo')" }}>
            </div>
            <span className="absolute bottom-0 right-0 block h-2.5 w-2.5 rounded-full bg-green-400 ring-2 ring-white dark:ring-surface-dark"></span>
          </div>
          <div className="relative inline-block">
            <div className="w-8 h-8 rounded-full border-2 border-white dark:border-surface-dark bg-cover bg-center bg-gray-400"
              style={{ backgroundImage: "url('https://lh3.googleusercontent.com/aida-public/AB6AXuCxqA75PIa4jEEKbnyU-w3meisZDCvhwPr-IwzlJAPSFiT1bgupilQB_RlmQ-IoaT5raPPAblNuyMc5rCOXx3TqDL-JxhcIYo7-DKRXjW6lJOkcYxpJiFkL89PzKlcIzllfUSqQdZHEC8u-uiSG7oCnZYRJyucDuSlwP66YK-azs2LcFwLz6_Wjjpr6TPfYdYcRUGUuCvRDjiWqOdSeaP2VxdLmORNfWYhqSwEctKFFAB1GQJnBFApPWfjah-20EgO-H7nPk-Qgujk')" }}>
            </div>
          </div>
          <div className="flex items-center justify-center w-8 h-8 rounded-full border-2 border-white dark:border-surface-dark bg-primary/20 text-xs font-bold text-primary">
            +3
          </div>
        </div>

        <button className="bg-primary hover:bg-sky-400 text-white dark:text-background-dark px-4 py-1.5 rounded-lg text-sm font-bold flex items-center gap-2 shadow-sm shadow-primary/30 transition-all">
          <Rocket className="w-[18px] h-[18px]" />
          <span>Deploy</span>
        </button>
      </div>
    </header>
  );
}
