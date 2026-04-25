---
name: Archa
colors:
  surface: '#faf8ff'
  surface-dim: '#d9d9e5'
  surface-bright: '#faf8ff'
  surface-container-lowest: '#ffffff'
  surface-container-low: '#f3f3fe'
  surface-container: '#ededf9'
  surface-container-high: '#e7e7f3'
  surface-container-highest: '#e1e2ed'
  on-surface: '#191b23'
  on-surface-variant: '#434655'
  inverse-surface: '#2e3039'
  inverse-on-surface: '#f0f0fb'
  outline: '#737686'
  outline-variant: '#c3c6d7'
  surface-tint: '#0053db'
  primary: '#004ac6'
  on-primary: '#ffffff'
  primary-container: '#2563eb'
  on-primary-container: '#eeefff'
  inverse-primary: '#b4c5ff'
  secondary: '#565e74'
  on-secondary: '#ffffff'
  secondary-container: '#dae2fd'
  on-secondary-container: '#5c647a'
  tertiary: '#943700'
  on-tertiary: '#ffffff'
  tertiary-container: '#bc4800'
  on-tertiary-container: '#ffede6'
  error: '#ba1a1a'
  on-error: '#ffffff'
  error-container: '#ffdad6'
  on-error-container: '#93000a'
  primary-fixed: '#dbe1ff'
  primary-fixed-dim: '#b4c5ff'
  on-primary-fixed: '#00174b'
  on-primary-fixed-variant: '#003ea8'
  secondary-fixed: '#dae2fd'
  secondary-fixed-dim: '#bec6e0'
  on-secondary-fixed: '#131b2e'
  on-secondary-fixed-variant: '#3f465c'
  tertiary-fixed: '#ffdbcd'
  tertiary-fixed-dim: '#ffb596'
  on-tertiary-fixed: '#360f00'
  on-tertiary-fixed-variant: '#7d2d00'
  background: '#faf8ff'
  on-background: '#191b23'
  surface-variant: '#e1e2ed'
typography:
  nav-item:
    fontFamily: Inter
    fontSize: 13px
    fontWeight: '500'
    lineHeight: 20px
  list-title:
    fontFamily: Inter
    fontSize: 14px
    fontWeight: '600'
    lineHeight: 18px
  list-preview:
    fontFamily: Inter
    fontSize: 12px
    fontWeight: '400'
    lineHeight: 16px
  reader-h1:
    fontFamily: Inter
    fontSize: 28px
    fontWeight: '700'
    lineHeight: 36px
    letterSpacing: -0.02em
  reader-body:
    fontFamily: Newsreader
    fontSize: 18px
    fontWeight: '400'
    lineHeight: 28px
  reader-code:
    fontFamily: Space Grotesk
    fontSize: 14px
    fontWeight: '400'
    lineHeight: 22px
  meta-label:
    fontFamily: Inter
    fontSize: 11px
    fontWeight: '600'
    lineHeight: 14px
    letterSpacing: 0.05em
rounded:
  sm: 0.125rem
  DEFAULT: 0.25rem
  md: 0.375rem
  lg: 0.5rem
  xl: 0.75rem
  full: 9999px
spacing:
  margin-page: 24px
  gutter-columns: 0px
  sidebar-width: 240px
  list-width: 320px
  reader-max-width: 800px
  unit: 4px
---

## Brand & Style

The brand personality of this design system is one of academic precision and industrial utility. It is designed for researchers, developers, and power users who treat AI interactions as a form of literature or data to be studied. The aesthetic is rooted in **Minimalism** with a heavy emphasis on structural clarity, drawing inspiration from high-end IDEs and technical documentation sites. 

The goal is to evoke a sense of focus and objectivity. By removing decorative elements and relying on strict grid alignment and subtle value shifts, the interface recedes into the background, allowing the conversation content—the "data"—to take center stage. The emotional response is one of calm productivity and intellectual rigour.

## Colors

This design system utilizes a high-key, "Paper & Ink" palette. The primary color is a precise blue, used sparingly for active states, commit buttons, or critical indicators to avoid visual fatigue in dense information environments.

The neutral palette is the backbone of the system. We use a range of cool-toned grays to define structural hierarchy. 
- **Main Surfaces:** Pure white (#FFFFFF) is reserved for the primary reading area to ensure maximum contrast.
- **Sidebars & Header:** A subtle gray (#F9FAFB) distinguishes navigation and utility areas from the main content.
- **Separators:** Hairline borders (#E5E7EB) are the primary tool for layout separation, replacing shadows to maintain a flat, tool-like feel.

## Typography

The typography strategy employs a dual-tone approach to differentiate between the "Container" (UI) and the "Content" (Data).

1.  **UI Elements (Inter):** A clean, highly legible sans-serif used for navigation, sidebar lists, property labels, and buttons. It communicates systematic reliability.
2.  **Reading Pane (Newsreader):** A sophisticated serif for the main AI conversation text. This creates an editorial feel that reduces eye strain during long-form reading and distinguishes the user's thoughts from the interface.
3.  **Technical Snippets (Space Grotesk):** A geometric monospace used for code blocks or technical metadata, providing a "futuristic" touch that aligns with the AI-centric nature of the product.

Hierarchy is established through tight control of line-height and letter-spacing, particularly in the metadata sections where information density is high.

## Layout & Spacing

This design system uses a **Fixed-Column Grid** layout inspired by professional IDEs. The screen is divided into three distinct vertical zones:
- **Primary Nav (Global):** Leftmost slim bar for high-level categories.
- **Object List (Collection):** Secondary column for browsing conversation history.
- **Reading View (Detail):** The central workspace, with an optional right-side "Inspector" pane for metadata.

Separation is achieved through 1px solid borders rather than gutters or margins, maximizing the screen real estate for content. Inside the reading view, content is centered with a max-width of 800px to maintain optimal line length for the serif body text. A strict 4px baseline grid ensures vertical rhythm across all columns.

## Elevation & Depth

To maintain a "Professional Tool" aesthetic, this design system avoids traditional drop shadows and physical metaphors. Depth is communicated through **Tonal Layering** and **Low-Contrast Outlines**.

- **Level 0 (Base):** The background of the application in a subtle gray.
- **Level 1 (Panes):** Content areas use pure white to appear "above" the base frame.
- **Level 2 (Overlays):** Context menus and command palettes use a very soft, diffused shadow (0px 4px 12px rgba(0,0,0,0.05)) and a 1px border to differentiate them from the background.

Backdrop blurs are used exclusively for floating headers in the reading pane to maintain context while scrolling without creating visual clutter.

## Shapes

The shape language is disciplined and rectangular. We use a "Soft" roundedness level (0.25rem) to take the edge off the industrial layout without making it feel "bubbly" or consumer-grade.

- **Primary Buttons:** Subtle rounding (4px) to match the grid.
- **Input Fields:** Crisp 4px corners with inset borders.
- **Tags/Chips:** Slightly more rounded (12px) to distinguish them from interactive buttons and indicate they are discrete data points.
- **Pane Transitions:** Sharp 90-degree corners where panes meet, reinforcing the structural, tool-like grid.

## Components

### Buttons
Buttons are predominantly "Ghost" or "Outline" style. The primary action button uses a solid fill with the primary blue, while secondary actions use a light gray hover state with no border. Text is always centered and uses the `nav-item` type spec.

### List Items
List items in the secondary column are high-density. They include a title, a two-line preview, and a timestamp. The active state is indicated by a subtle background shift (#F3F4F6) and a 2px vertical accent bar on the left edge.

### Metadata Chips
Used for "Status," "Model," or "Tags." These should have a very light background fill (e.g., #EFF6FF for blue) and text in a darkened version of that color. They use the `meta-label` typography spec.

### Reading Pane
The core component. It must support Markdown rendering. Headers use Inter (Sans), while body text transitions to Newsreader (Serif). Code blocks use a light gray background with no border and Space Grotesk.

### Inspector Sidebar
A vertical list of key-value pairs using `meta-label` for keys and `nav-item` for values. Properties are separated by thin horizontal rules.