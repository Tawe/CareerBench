import { useEffect, useState, useRef } from "react";
import "./QuickSwitcher.css";

interface QuickSwitcherItem {
  id: string;
  label: string;
  description?: string;
  action: () => void;
  category: "navigation" | "action";
}

interface QuickSwitcherProps {
  isOpen: boolean;
  onClose: () => void;
  items: QuickSwitcherItem[];
}

export function QuickSwitcher({ isOpen, onClose, items }: QuickSwitcherProps) {
  const [selectedIndex, setSelectedIndex] = useState(0);
  const [searchQuery, setSearchQuery] = useState("");
  const inputRef = useRef<HTMLInputElement>(null);

  const filteredItems = items.filter((item) =>
    item.label.toLowerCase().includes(searchQuery.toLowerCase()) ||
    item.description?.toLowerCase().includes(searchQuery.toLowerCase())
  );

  useEffect(() => {
    if (isOpen) {
      setSearchQuery("");
      setSelectedIndex(0);
      // Focus input after a brief delay to ensure it's rendered
      setTimeout(() => {
        inputRef.current?.focus();
      }, 50);
    }
  }, [isOpen]);

  useEffect(() => {
    if (!isOpen) return;

    function handleKeyDown(e: KeyboardEvent) {
      if (e.key === "Escape") {
        onClose();
      } else if (e.key === "ArrowDown") {
        e.preventDefault();
        setSelectedIndex((prev) => 
          prev < filteredItems.length - 1 ? prev + 1 : prev
        );
      } else if (e.key === "ArrowUp") {
        e.preventDefault();
        setSelectedIndex((prev) => (prev > 0 ? prev - 1 : 0));
      } else if (e.key === "Enter") {
        e.preventDefault();
        if (filteredItems[selectedIndex]) {
          filteredItems[selectedIndex].action();
          onClose();
        }
      }
    }

    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [isOpen, selectedIndex, onClose, filteredItems]);

  useEffect(() => {
    if (selectedIndex >= filteredItems.length) {
      setSelectedIndex(Math.max(0, filteredItems.length - 1));
    }
  }, [filteredItems.length, selectedIndex]);

  if (!isOpen) return null;

  return (
    <>
      <div className="quick-switcher-overlay" onClick={onClose}></div>
      <div className="quick-switcher-container" onClick={(e) => e.stopPropagation()}>
        <div className="quick-switcher-header">
          <input
            ref={inputRef}
            type="text"
            placeholder="Type to search..."
            value={searchQuery}
            onChange={(e) => {
              setSearchQuery(e.target.value);
              setSelectedIndex(0);
            }}
            className="quick-switcher-input"
          />
        </div>
        <div className="quick-switcher-results">
          {filteredItems.length === 0 ? (
            <div className="quick-switcher-empty">No results found</div>
          ) : (
            filteredItems.map((item, index) => (
              <div
                key={item.id}
                className={`quick-switcher-item ${
                  index === selectedIndex ? "selected" : ""
                }`}
                onClick={() => {
                  item.action();
                  onClose();
                }}
                onMouseEnter={() => setSelectedIndex(index)}
              >
                <div className="quick-switcher-item-content">
                  <div className="quick-switcher-item-label">{item.label}</div>
                  {item.description && (
                    <div className="quick-switcher-item-description">
                      {item.description}
                    </div>
                  )}
                </div>
                <div className="quick-switcher-item-category">
                  {item.category === "navigation" ? "→" : "⚡"}
                </div>
              </div>
            ))
          )}
        </div>
        <div className="quick-switcher-footer">
          <div className="quick-switcher-hint">
            <kbd>↑</kbd>
            <kbd>↓</kbd> Navigate
            <kbd>Enter</kbd> Select
            <kbd>Esc</kbd> Close
          </div>
        </div>
      </div>
    </>
  );
}

