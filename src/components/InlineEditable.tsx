import { useState, useRef, useEffect } from "react";
import "./InlineEditable.css";

interface InlineEditableProps {
  value: string;
  onSave: (value: string) => void | Promise<void>;
  placeholder?: string;
  className?: string;
  multiline?: boolean;
  rows?: number;
  disabled?: boolean;
}

export function InlineEditable({
  value,
  onSave,
  placeholder = "Click to edit",
  className = "",
  multiline = false,
  rows = 1,
  disabled = false,
}: InlineEditableProps) {
  const [isEditing, setIsEditing] = useState(false);
  const [editValue, setEditValue] = useState(value);
  const inputRef = useRef<HTMLInputElement | HTMLTextAreaElement>(null);

  useEffect(() => {
    setEditValue(value);
  }, [value]);

  useEffect(() => {
    if (isEditing && inputRef.current) {
      inputRef.current.focus();
      if (inputRef.current instanceof HTMLInputElement || inputRef.current instanceof HTMLTextAreaElement) {
        inputRef.current.select();
      }
    }
  }, [isEditing]);

  function handleStartEdit() {
    if (disabled) return;
    setIsEditing(true);
    setEditValue(value);
  }

  async function handleSave() {
    if (editValue !== value) {
      await onSave(editValue);
    }
    setIsEditing(false);
  }

  function handleCancel() {
    setEditValue(value);
    setIsEditing(false);
  }

  function handleKeyDown(e: React.KeyboardEvent) {
    if (e.key === "Enter" && !multiline) {
      e.preventDefault();
      handleSave();
    } else if (e.key === "Enter" && multiline && (e.metaKey || e.ctrlKey)) {
      e.preventDefault();
      handleSave();
    } else if (e.key === "Escape") {
      e.preventDefault();
      handleCancel();
    }
  }

  function handleBlur() {
    handleSave();
  }

  if (isEditing) {
    if (multiline) {
      return (
        <textarea
          ref={inputRef as React.RefObject<HTMLTextAreaElement>}
          value={editValue}
          onChange={(e) => setEditValue(e.target.value)}
          onKeyDown={handleKeyDown}
          onBlur={handleBlur}
          rows={rows}
          className={`inline-editable-input ${className}`}
          placeholder={placeholder}
        />
      );
    } else {
      return (
        <input
          ref={inputRef as React.RefObject<HTMLInputElement>}
          type="text"
          value={editValue}
          onChange={(e) => setEditValue(e.target.value)}
          onKeyDown={handleKeyDown}
          onBlur={handleBlur}
          className={`inline-editable-input ${className}`}
          placeholder={placeholder}
        />
      );
    }
  }

  return (
    <span
      onClick={handleStartEdit}
      onDoubleClick={handleStartEdit}
      className={`inline-editable-display ${className}`}
      style={{
        cursor: disabled ? "default" : "text",
        minHeight: multiline ? "1.5rem" : "1.25rem",
        display: "inline-block",
        width: "100%",
        padding: "0.25rem 0.5rem",
        borderRadius: "0.25rem",
        transition: "background-color 0.15s ease",
      }}
      onMouseEnter={(e) => {
        if (!disabled) {
          e.currentTarget.style.backgroundColor = "#f3f4f6";
        }
      }}
      onMouseLeave={(e) => {
        e.currentTarget.style.backgroundColor = "transparent";
      }}
    >
      {value || <span style={{ color: "#9ca3af", fontStyle: "italic" }}>{placeholder}</span>}
    </span>
  );
}

