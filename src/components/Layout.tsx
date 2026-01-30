import { Link, useLocation, useNavigate } from "react-router-dom";
import { useState, useEffect } from "react";
import { QuickSwitcher } from "./QuickSwitcher";
import { ToastContainer, useToasts } from "./Toast";
import "./Layout.css";
import "./QuickSwitcher.css";
import "./Toast.css";

interface LayoutProps {
  children: React.ReactNode;
}

export default function Layout({ children }: LayoutProps) {
  const location = useLocation();
  const navigate = useNavigate();
  const [isCollapsed, setIsCollapsed] = useState(false);
  const [showQuickSwitcher, setShowQuickSwitcher] = useState(false);
  const { toasts, removeToast } = useToasts();

  const navItems = [
    { 
      path: "/", 
      label: "Dashboard",
      icon: (
        <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
          <rect x="3" y="3" width="7" height="7"></rect>
          <rect x="14" y="3" width="7" height="7"></rect>
          <rect x="14" y="14" width="7" height="7"></rect>
          <rect x="3" y="14" width="7" height="7"></rect>
        </svg>
      )
    },
    { 
      path: "/jobs", 
      label: "Jobs",
      icon: (
        <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
          <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"></path>
          <polyline points="14 2 14 8 20 8"></polyline>
          <line x1="16" y1="13" x2="8" y2="13"></line>
          <line x1="16" y1="17" x2="8" y2="17"></line>
          <polyline points="10 9 9 9 8 9"></polyline>
        </svg>
      )
    },
    { 
      path: "/applications", 
      label: "Applications",
      icon: (
        <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
          <path d="M9 11l3 3L22 4"></path>
          <path d="M21 12v7a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h11"></path>
        </svg>
      )
    },
    { 
      path: "/calendar", 
      label: "Calendar",
      icon: (
        <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
          <rect x="3" y="4" width="18" height="18" rx="2" ry="2"></rect>
          <line x1="16" y1="2" x2="16" y2="6"></line>
          <line x1="8" y1="2" x2="8" y2="6"></line>
          <line x1="3" y1="10" x2="21" y2="10"></line>
        </svg>
      )
    },
    { 
      path: "/learning", 
      label: "Learning",
      icon: (
        <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
          <path d="M2 3h6a4 4 0 0 1 4 4v14a3 3 0 0 0-3-3H2z"></path>
          <path d="M22 3h-6a4 4 0 0 0-4 4v14a3 3 0 0 1 3-3h7z"></path>
        </svg>
      )
    },
    { 
      path: "/recruiters", 
      label: "Recruiters",
      icon: (
        <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
          <path d="M17 21v-2a4 4 0 0 0-4-4H5a4 4 0 0 0-4 4v2"></path>
          <circle cx="9" cy="7" r="4"></circle>
          <path d="M23 21v-2a4 4 0 0 0-3-3.87"></path>
          <path d="M16 3.13a4 4 0 0 1 0 7.75"></path>
        </svg>
      )
    },
    { 
      path: "/companies", 
      label: "Companies",
      icon: (
        <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
          <path d="M3 9l9-7 9 7v11a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2z"></path>
          <polyline points="9 22 9 12 15 12 15 22"></polyline>
        </svg>
      )
    },
    { 
      path: "/profile", 
      label: "Profile",
      icon: (
        <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
          <path d="M20 21v-2a4 4 0 0 0-4-4H8a4 4 0 0 0-4 4v2"></path>
          <circle cx="12" cy="7" r="4"></circle>
        </svg>
      )
    },
    { 
      path: "/settings", 
      label: "Settings",
      icon: (
        <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
          <circle cx="12" cy="12" r="3"></circle>
          <path d="M12 1v6m0 6v6m9-9h-6m-6 0H3"></path>
        </svg>
      )
    },
  ];

  // Keyboard shortcut handler for Cmd+K / Ctrl+K
  useEffect(() => {
    function handleKeyDown(e: KeyboardEvent) {
      // Check for Cmd+K (Mac) or Ctrl+K (Windows/Linux)
      if ((e.metaKey || e.ctrlKey) && e.key === "k") {
        e.preventDefault();
        setShowQuickSwitcher(true);
      }
    }

    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, []);

  const quickSwitcherItems = [
    {
      id: "dashboard",
      label: "Go to Dashboard",
      description: "View your application dashboard",
      category: "navigation" as const,
      action: () => navigate("/"),
    },
    {
      id: "jobs",
      label: "Go to Jobs",
      description: "View and manage job listings",
      category: "navigation" as const,
      action: () => navigate("/jobs"),
    },
    {
      id: "applications",
      label: "Go to Applications",
      description: "View and manage your applications",
      category: "navigation" as const,
      action: () => navigate("/applications"),
    },
    {
      id: "calendar",
      label: "Go to Calendar",
      description: "View interviews and upcoming events",
      category: "navigation" as const,
      action: () => navigate("/calendar"),
    },
    {
      id: "profile",
      label: "Go to Profile",
      description: "Edit your profile information",
      category: "navigation" as const,
      action: () => navigate("/profile"),
    },
    {
      id: "learning",
      label: "Go to Learning",
      description: "View learning plans and skill gaps",
      category: "navigation" as const,
      action: () => navigate("/learning"),
    },
    {
      id: "recruiters",
      label: "Go to Recruiters",
      description: "Manage recruiter contacts and interactions",
      category: "navigation" as const,
      action: () => navigate("/recruiters"),
    },
    {
      id: "companies",
      label: "Go to Companies",
      description: "Manage company information and links",
      category: "navigation" as const,
      action: () => navigate("/companies"),
    },
    {
      id: "settings",
      label: "Go to Settings",
      description: "Configure application settings",
      category: "navigation" as const,
      action: () => navigate("/settings"),
    },
  ];

  return (
    <div className="layout">
      <nav className={`sidebar ${isCollapsed ? "collapsed" : ""}`}>
        <div className="sidebar-header">
          {!isCollapsed && <h1>CareerBench</h1>}
          <button 
            className="sidebar-toggle"
            onClick={() => setIsCollapsed(!isCollapsed)}
            aria-label={isCollapsed ? "Expand sidebar" : "Collapse sidebar"}
          >
            <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
              {isCollapsed ? (
                <polyline points="9 18 15 12 9 6"></polyline>
              ) : (
                <polyline points="15 18 9 12 15 6"></polyline>
              )}
            </svg>
          </button>
        </div>
        <ul className="nav-list">
          {navItems.map((item) => (
            <li key={item.path}>
              <Link
                to={item.path}
                className={location.pathname === item.path ? "active" : ""}
                title={isCollapsed ? item.label : undefined}
              >
                <span className="nav-icon">{item.icon}</span>
                {!isCollapsed && <span className="nav-label">{item.label}</span>}
              </Link>
            </li>
          ))}
        </ul>
      </nav>
      <main className={`main-content ${isCollapsed ? "collapsed" : ""}`}>
        <div className="max-w-5xl mx-auto">
          {children}
        </div>
      </main>
      <QuickSwitcher
        isOpen={showQuickSwitcher}
        onClose={() => setShowQuickSwitcher(false)}
        items={quickSwitcherItems}
      />
      <ToastContainer toasts={toasts} onClose={removeToast} />
    </div>
  );
}

