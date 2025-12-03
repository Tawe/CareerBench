# CareerBench UI Design Specification

This specification defines the **visual language**, **interaction patterns**, and **component system** for the CareerBench Tauri desktop application. It is heavily inspired by **Notion's minimalist, document-centric design**, adapted to a job-search productivity tool.

The purpose of this spec is to guide implementation in **React + TailwindCSS + shadcn/ui (Radix)** within Tauri.

---

# 1. Design Principles

CareerBench should feel:

### **Calm**

A neutral, low-contrast interface where content—not chrome—holds attention.

### **Focused**

Document-like surfaces, minimal borders, lots of whitespace.

### **Effortless**

Interactions appear only when needed: hover menus, inline editing, lightweight dialogs.

### **Structured**

Information is organized into blocks: cards, list rows, text sections—similar to Notion’s content patterns.

---

# 2. Color System

CareerBench uses a **soft, neutral palette** with subtle accents.

## **2.1 Core Palette**

|Token|Hex|Usage|
|---|---|---|
|`--cb-bg`|`#fafafa`|App background, main canvas|
|`--cb-bg-elevated`|`#ffffff`|Cards, panels, modals|
|`--cb-border`|`#e5e7eb`|Subtle dividers, outlines|
|`--cb-text`|`#111827`|Primary text|
|`--cb-text-muted`|`#6b7280`|Metadata, labels|
|`--cb-accent`|`#6366f1`|Indigo accent for buttons, highlights|
|`--cb-accent-soft`|`#eef2ff`|Soft indigo background|

## **2.2 Status Colors**

Soft, Notion-style badges.

|   |   |   |
|---|---|---|
|Status|Background|Text|
|Saved|`#f3f4f6`|`#374151`|
|Applied|`#dbeafe`|`#1e40af`|
|Interviewing|`#ede9fe`|`#5b21b6`|
|Offer|`#d1fae5`|`#065f46`|
|Rejected|`#fee2e2`|`#991b1b`|
|Ghosted|`#e5e7eb`|`#374151`|
|Withdrawn|`#f3f4f6`|`#374151`|

---

# 3. Typography

Notion-like typography is clean, readable, and rarely bold.

### **Font Family**

```
Inter, system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Helvetica, sans-serif
```

### **Font Sizes**

- `text-xs` (metadata)
    
- `text-sm` (dense lists)
    
- `text-base` (body)
    
- `text-lg` (section headers)
    
- `text-xl` (page titles)
    
- `text-2xl` (module titles)
    

### **Line Height**

- Body: `leading-relaxed`
    
- Titles: `leading-tight`
    

### **Weight**

- Mostly `font-normal`
    
- Titles: `font-medium`
    
- Rarely use `font-semibold`
    

---

# 4. Spacing & Layout

CareerBench mirrors Notion’s comfortable whitespace.

### **Global Spacing Scale**

- `4px` → tight content
    
- `8px` → metadata
    
- `12px` → rows
    
- `16px` → section spacing
    
- `24px` → page spacing
    
- `32px` → major content blocks
    

### **Radius & Shadow**

- `rounded-lg` on cards/panels
    
- `rounded-md` on form elements
    
- Shadows: minimal
    
    - `shadow-sm` or none
        
    - Prefer borders over shadows
        

---

# 5. Core Layout Components

These are the reusable layout elements that define the app.

## **5.1 AppShell**

The structure for every page.

```
+--------------------------------------------+
| Sidebar | Content Area                     |
|         |                                  |
|         |                                  |
+--------------------------------------------+
```

- Sidebar (left, fixed width: 240px)
    
- Content area (scrollable)
    
- Background: `bg-[--cb-bg]`
    

## **5.2 Sidebar**

- Notion-style vertical nav
    
- Sections:
    
    - App logo/name
        
    - Main nav: Dashboard, Jobs, Applications, Learning, Settings
        
- Use `hover:bg-gray-100` and subtle indicators
    

## **5.3 Top Bar (optional)**

- Only for pages needing search or dense actions
    
- Thin bar with minimal chrome
    

## **5.4 Page Header**

Clean header with:

- Title (e.g., "Applications")
    
- Subtitle (optional)
    
- Right-aligned actions (lightweight buttons, dropdowns)
    

## **5.5 Page Body**

Document-like formatting:

- `max-w-5xl mx-auto`
    
- Generous spacing between sections
    
- Blocks for lists, details, charts, or forms
    

---

# 6. Component Patterns

**Note:** CareerBench avoids disruptive modals. All creation/edit flows use **right-side sheets (slide-over panels)** or **inline editing**, following Notion/Linear patterns.

## **6.1 Cards (Notion-style)**

- Soft borders: `border border-[--cb-border]`
    
- White background
    
- `rounded-lg`
    
- Minimal shadow or none
    
- Use for:
    
    - KPI metrics
        
    - Job summaries
        
    - Interview blocks
        

## **6.2 Lists**

### Job list / Application list

- Clean rows with:
    
    - Title
        
    - Subtext (company, location, date)
        
    - Soft status badge
        
- Row hover: `bg-gray-50`
    
- Use inline controls via `DropdownMenu` (three-dot menu appears on hover)
    

## **6.3 Detail Pages**

Document-style content blocks:

- Title
    
- Metadata row
    
- Section blocks:
    
    - Description
        
    - Responsibilities
        
    - Required skills
        
    - Parsed AI insights
        
- Divider: 1px subtle border or extra spacing
    

## **6.4 Tabs**

Used in Application Detail:

- Overview
    
- Timeline
    
- Artifacts
    
- Notes
    

Use shadcn/ui `Tabs` but styled lightly.

## **6.5 Drawers / Panels**

Right-side panels for editing:

- Resume preview
    
- Artifact text editor
    
- AI suggestions
    

Triggered with soft animations.

## **6.6 Badges**

Soft, rounded pills for statuses.

Example:

```
<span class="px-2 py-1 text-xs rounded-full bg-blue-100 text-blue-700">
  Applied
</span>
```

## **6.7 Buttons**

Minimal, like Notion:

### Primary

```
px-4 py-2 rounded-md bg-[--cb-accent] text-white hover:bg-indigo-600
```

### Secondary

```
px-4 py-2 rounded-md border border-[--cb-border] bg-white hover:bg-gray-50
```

### Ghost

```
p-2 rounded-md hover:bg-gray-100
```

Used for: small actions, menu toggles.

---

# 7. Animation & Interaction

### **Sheet Behavior (replacing traditional modals)**

- Sheets slide in from the right (`ease-out` 200–250ms).
    
- Sheets do **not block context** — background remains visible.
    
- Sheets are used for:
    
    - Creating or editing jobs
        
    - Creating or editing applications
        
    - Editing profile entries
        
    - Resume & cover letter generation
        
    - Viewing/editing artifacts
        
- Backdrop should be subtle or removed to maintain workspace continuity.
    

CareerBench should feel smooth without being flashy.

### **7.1 Motion**

- Use subtle transitions:
    
    - `transition-colors`
        
    - `transition-opacity`
        
    - `transition-transform`
        
- Panels slide in gently (`ease-out`, 200–250ms)
    

### **7.2 Hover Behavior**

- Inline controls appear on hover
    
- Row hovers are `bg-gray-50`
    
- Three-dot menus appear only when needed
    

### **7.3 Inline Editing**

Notion-inspired edit behavior:

- Titles become inputs when clicked
    
- Status can be changed inline via dropdown
    
- Notes editable in place
    

---

# 8. Page Templates

## **8.1 Dashboard Page**

- Page header: "Dashboard"
    
- KPI card row
    
- Status overview (badge list or small chart)
    
- Funnel (three-bar vertical)
    
- Activity chart (Recharts)
    

## **8.2 Jobs Page**

Left column: job list Right column: job detail

Detail page sections:

- Title @ Company
    
- Metadata row
    
- Parsed AI summary
    
- Responsibilities
    
- Skills
    
- Action buttons (resume generator, create application)
    

## **8.3 Applications Page**

- Filter pills across the top
    
- Application list (database-like rows)
    
- Detail page with tabs:
    
    - Overview
        
    - Timeline
        
    - Artifacts
        
    - Notes
        

## **8.4 Learning Plan Page**

- AI-generated roadmap blocks
    
- Tasks list (like Notion checklists)
    
- Progress indicators
    

---

# 9. Component Library Mapping (shadcn/ui)

Use these components to construct CareerBench:

|   |   |
|---|---|
|UI Concept|shadcn Component|
|Page layout|Custom + Tailwind|
|Sidebar|Custom div + nav styling|
|Page header|Flex rows + Button, DropdownMenu|
|Cards|Card|
|Badges|Badge|
|Sheets \| Sheet (slide-over panels)|
|Panels|Sheet (Right-side)|
|Tabs|Tabs|
|Dropdown menus|DropdownMenu|
|Inputs, textareas|Input, Textarea|
|Selects|Select|
|Tabs for artifacts|Tabs|
|Toasts|Sonner (optional)|

---

# 10. Implementation Notes for Cursor

### **10.1 Use Tailwind tokens for colors**

Define CSS variables in `globals.css`:

```
:root {
  --cb-bg: #fafafa;
  --cb-bg-elevated: #ffffff;
  --cb-border: #e5e7eb;
  --cb-text: #111827;
  --cb-text-muted: #6b7280;
  --cb-accent: #6366f1;
  --cb-accent-soft: #eef2ff;
}
```

### **10.2 Layout Architecture**

Create shared components:

- `AppShell.tsx`
    
- `Sidebar.tsx`
    
- `PageHeader.tsx`
    
- `Card.tsx` (wrap shadcn)
    
- `StatusBadge.tsx`
    
- `DetailSection.tsx`
    
- `EditorPanel.tsx` (right drawer)
    

### **10.3 Reusable Patterns**

- List rows with inline menus
    
- Soft bordered cards for analytics
    
- Right-panel editors for resume artifacts
    
- Tabbed layouts for detail pages
    

---

# 11. Interaction Patterns & User Behaviors

This section describes how users should _feel_ when interacting with CareerBench. These patterns take inspiration from Notion’s balance of minimal UI, smooth interactions, and inline editing.

---

## **11.1 Interaction Modes**

CareerBench supports three core interaction modes:

### **1) Browsing Mode**

Users are reading, skimming, or comparing.

- Lightweight hover states (`bg-gray-50`).
    
- Inline metadata surfaces (company, dates).
    
- Search + filter drawers to help narrow focus.
    

### **2) Editing Mode**

Users are actively updating content.

- Click-to-edit inline fields.
    
- Auto-save on blur.
    
- ESC to cancel, CMD+Enter to confirm.
    
- Inputs appear gracefully—no jarring transitions.
    

### **3) Action Mode**

Triggered when performing major tasks.

- Modals or sheets (shadcn `Dialog`, `Sheet`).
    
- Single primary button.
    
- Clear, linear flow (Options → Preview → Save).
    

---

## **11.2 Navigation Patterns**

CareerBench should feel like a workspace, not a website.

### **Sidebar Navigation**

- Persistent left sidebar with Dashboard, Jobs, Applications, Learning, Settings.
    
- Currently selected item uses a soft highlight or left border.
    
- Clicking does NOT reload—content swaps smoothly.
    

### **List + Detail Pattern**

Almost all modules follow this structure:

```
[ LIST OF ITEMS ]    [ DETAILS OF SELECTED ITEM ]
```

This keeps context visible and reduces navigation friction.

---

## **11.3 Inline Controls & Hover-First UI**

Inspired by Notion’s minimalist approach.

### Lists

- Three-dot menus appear on hover.
    
- Status badges clickable inline.
    
- Notes previews appear on hover.
    

### Detail Sections

- Section headers show edit icon on hover.
    
- Inline AI actions appear next to editable text.
    

---

## **11.4 Module-Specific Interaction Patterns**

### **Jobs Module**

- Clicking a job opens detail on right.
    
- Hover controls: Parse with AI, Create Application.
    
- Inline editing for title, tags, metadata.
    
- Long descriptions collapse/expand elegantly.
    

### **Applications Module**

- Status change is inline via dropdown menu.
    
- Timeline entries have + buttons on hover.
    
- Tabs (Overview, Timeline, Artifacts, Notes).
    
- Artifact cards show controls on hover (Open, Regenerate).
    

### **Resume & Cover Letter Generator**

- Opens in a right-side sheet.
    
- Options → AI Generation → Editable Preview.
    
- Save creates artifact + auto-links to Job/Application.
    
- User can edit text in place.
    

### **Learning Plan Module**

- AI-generated blocks.
    
- Inline editing for learning goals.
    
- Task lists with checkboxes.
    
- Drag-and-drop ordering (future enhancement).
    

---

## **11.5 Keyboard Shortcuts**

Lightweight productivity features:

### Global

- `Cmd+K` — Quick switcher.
    
- `/` — Slash menu in notes (future).
    

### Lists

- Up/Down arrows to move between rows.
    
- Enter to open detail.
    
- Backspace to delete (with confirmation).
    

### Forms

- `Cmd+Enter` to save.
    
- `Esc` to cancel.
    

### Modals

- Enter triggers primary action.
    
- ESC closes.
    

---

## **11.6 Feedback Patterns**

### Skeleton Loading

- Shimmer placeholders for job parsing, resume generation, dashboard metrics.
    

### Optimistic Updates

- Inline edits update UI immediately.
    
- Small checkmark or subtle toast when successful.
    

### Errors

- Soft red outline for invalid fields.
    
- Toast messages bottom-left.
    

---

# 12. Future Enhancements

- Theme switcher (light/dark mode)
    
- Notion-style slash command menu (`/`)
    
- Drag-and-drop reordering for learning tasks
    
- Contextual AI actions (hover-based “Ask AI”)
    

---

# 13. Summary

This UI spec defines a clean, calm, Notion-inspired aesthetic for CareerBench. It ensures consistency across modules, provides clear implementation patterns for Cursor, and supports scalability as new features (learning plans, AI assistants) are added.

Use this design spec to generate:

- Layout scaffolds
    
- Component systems
    
- Tailwind theme
    
- Page templates
    

CareerBench should feel simple, elegant, and focused—just like Notion, but purpose-built for job search mastery.