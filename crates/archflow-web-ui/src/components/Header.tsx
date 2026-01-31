interface HeaderProps {
  darkMode: boolean;
  onToggleDarkMode: () => void;
}

export default function Header({ darkMode, onToggleDarkMode }: HeaderProps) {
  return (
    <header className="flex items-center justify-between whitespace-nowrap border-b border-solid border-border-light dark:border-border-dark bg-surface-light dark:bg-surface-dark px-6 py-2 shrink-0 z-30">
      <div className="flex items-center gap-6">
        {/* Logo */}
        <div className="flex items-center gap-2 text-[#0d181b] dark:text-white">
          <span className="material-symbols-outlined text-3xl">
            deployed_code
          </span>
          <h2 className="text-xl font-bold leading-tight tracking-[-0.015em]">
            ArchFlow
          </h2>
        </div>

        {/* Breadcrumbs */}
        <div className="hidden sm:flex items-center gap-2 text-sm">
          <span className="text-slate-500 dark:text-slate-400 font-medium">
            Team Workspace
          </span>
          <span className="text-slate-300 dark:text-slate-600">/</span>
          <span className="text-slate-500 dark:text-slate-400 font-medium">
            Project Alpha
          </span>
          <span className="text-slate-300 dark:text-slate-600">/</span>
          <span className="text-[#0d181b] dark:text-white font-bold">
            Architecture V1
          </span>
        </div>
      </div>

      <div className="flex items-center gap-4">
        {/* Navigation */}
        <nav className="flex items-center gap-6 mr-4 hidden md:flex">
          <button className="text-slate-600 dark:text-slate-300 hover:text-primary transition-colors text-sm font-medium">
            File
          </button>
          <button className="text-slate-600 dark:text-slate-300 hover:text-primary transition-colors text-sm font-medium">
            Edit
          </button>
          <button className="text-slate-600 dark:text-slate-300 hover:text-primary transition-colors text-sm font-medium">
            View
          </button>
          <button className="text-slate-600 dark:text-slate-300 hover:text-primary transition-colors text-sm font-medium">
            Help
          </button>
        </nav>

        {/* Avatar Group */}
        <div className="flex -space-x-2">
          <div className="relative inline-block">
            <div
              className="size-8 rounded-full border-2 border-white dark:border-background-dark bg-cover bg-center"
              style={{
                backgroundImage:
                  'url("https://lh3.googleusercontent.com/aida-public/AB6AXuCZz2zC4QuZ416Ac930ywOaXh7SmLUaNX3JKzmPAaRIuZFX_EjCl5hcyLUgkfeWKYeA-kBwUY4eThcEOTT7SwJrpmoHsg137GyosICeQ7DGDaggGQ_nFq6JM3eVErz0VLrM8jnoh0ClYogZR7mVSjR3--zc8in9CmarDO9Z058bihLGyXR3LDhsqeIMFLjk3lzs1LG4hXGXsD4FIzuCd88QbefYJ7QYgi7OToYOnw6576Xn_NDSSb8yMfFAp2GAz7zN95OezN1YIXo")',
              }}
            />
            <span className="absolute bottom-0 right-0 block h-2.5 w-2.5 rounded-full bg-green-400 ring-2 ring-white dark:ring-background-dark"></span>
          </div>
          <div className="relative inline-block">
            <div
              className="size-8 rounded-full border-2 border-white dark:border-background-dark bg-cover bg-center"
              style={{
                backgroundImage:
                  'url("https://lh3.googleusercontent.com/aida-public/AB6AXuCxqA75PIa4jEEKbnyU-w3meisZDCvhwPr-IwzlJAPSFiT1bgupilQB_RlmQ-IoaT5raPPAblNuyMc5rCOXx3TqDL-JxhcIYo7-DKRXjW6lJOkcYxpJiFkL89PzKlcIzllfUSqQdZHEC8u-uiSG7oCnZYRJyucDuSlwP66YK-azs2LcFwLz6_Wjjpr6TPfYdYcRUGUuCvRDjiWqOdSeaP2VxdLmORNfWYhqSwEctKFFAB1GQJnBFApPWfjah-20EgO-H7nPk-Qgujk")',
              }}
            />
          </div>
          <div className="flex items-center justify-center size-8 rounded-full border-2 border-white dark:border-background-dark bg-primary/20 text-xs font-bold text-primary dark:text-primary-light">
            +3
          </div>
        </div>

        {/* Dark Mode Toggle */}
        <button
          onClick={onToggleDarkMode}
          className="flex items-center gap-1 text-slate-600 dark:text-slate-300 hover:text-primary transition-colors"
        >
          {darkMode ? (
            <span className="material-symbols-outlined">light_mode</span>
          ) : (
            <span className="material-symbols-outlined">dark_mode</span>
          )}
        </button>
      </div>
    </header>
  );
}
