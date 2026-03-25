# StellarEscrow Accessibility Documentation

## Overview

This document outlines the accessibility features implemented in the StellarEscrow dashboard to ensure WCAG 2.1 AA compliance and provide an inclusive user experience for all users, including those with disabilities.

## Table of Contents

1. [Accessibility Features](#accessibility-features)
2. [Keyboard Navigation](#keyboard-navigation)
3. [Screen Reader Support](#screen-reader-support)
4. [High Contrast Mode](#high-contrast-mode)
5. [Color Contrast](#color-contrast)
6. [Responsive Design](#responsive-design)
7. [Testing](#testing)
8. [WCAG Compliance Checklist](#wcag-compliance-checklist)

---

## Accessibility Features

### Semantic HTML Structure

The dashboard uses semantic HTML5 elements throughout:

- `<header role="banner">` - Site header with navigation
- `<nav role="navigation">` - Primary navigation menu
- `<main role="main">` - Primary content area
- `<footer role="contentinfo">` - Footer with links
- `<article>` - Self-contained content sections
- `<section>` - Thematic groupings of content
- `<form>` - Interactive forms with proper labels

### ARIA Landmarks

All major page regions are identified with ARIA roles:

| Region | ARIA Role | Purpose |
|--------|-----------|---------|
| Header | `role="banner"` | Site branding and navigation |
| Navigation | `role="navigation"` | Primary menu |
| Main Content | `role="main"` | Primary page content |
| Footer | `role="contentinfo"` | Footer information |
| Search | `role="search"` | Search functionality |
| Log | `role="log"` | Event feed updates |

### ARIA Labels

All interactive elements have descriptive labels:

- Navigation links with `aria-label` for context
- Buttons with accessible names
- Form inputs linked to labels via `for` attribute
- Icons include `aria-hidden="true"` to hide from screen readers
- Dynamic content announced via `aria-live` regions

---

## Keyboard Navigation

### Focus Management

All interactive elements are keyboard accessible:

| Element | Key | Action |
|---------|-----|--------|
| Skip Link | `Tab` | Jump to main content |
| Navigation | `Tab` / `Arrow keys` | Navigate between items |
| Buttons | `Enter` / `Space` | Activate button |
| Links | `Enter` | Follow link |
| Tables | `Arrow keys` | Navigate cells |
| FAQ Items | `Enter` / `Space` | Toggle expand/collapse |
| High Contrast Toggle | `Alt + H` | Toggle high contrast mode |
| Dropdowns | `Escape` | Close dropdown |

### Focus Indicators

Visible focus indicators are provided for all interactive elements:

- 3px solid outline on `:focus-visible`
- High contrast color for visibility
- Custom `--focus-ring` variable for consistent styling
- Inset focus for certain elements

### Skip Link

A "Skip to main content" link is available at the top of the page, visible only on keyboard focus. This allows users to bypass repetitive navigation.

```html
<a href="#main-content" class="skip-link">Skip to main content</a>
```

---

## Screen Reader Support

### Live Regions

Dynamic content changes are announced to screen readers:

| Region | `aria-live` | Update Type |
|--------|-------------|-------------|
| `#live-region` | `polite` | General announcements |
| `#events-list` | `polite` | Event feed updates |
| `#search-results` | `polite` | Search results |
| `#toast-container` | `assertive` | Error messages |

### Announcements

The `announce()` function provides screen reader notifications:

```javascript
announce('Event loaded successfully');
announce('High contrast mode enabled', 'assertive');
```

### Form Accessibility

All forms include:

- Visible `<label>` elements linked via `for` attribute
- `aria-describedby` for help text
- `aria-invalid` and `aria-errormessage` for validation errors
- Clear error messages in text format

### Table Accessibility

Data tables include:

- `<caption>` or `aria-describedby` for context
- `<th scope="col">` for column headers
- Keyboard navigation with arrow keys
- Focus indicators on cells

---

## High Contrast Mode

### Toggle Feature

Users can toggle high contrast mode via:

1. Button in bottom-right corner
2. Keyboard shortcut: `Alt + H`

### High Contrast Styles

When enabled:

- Pure black background (#000000)
- Pure white text (#ffffff)
- High saturation accent color (#00b4ff)
- Increased border visibility (#666666)
- Enhanced focus indicators

### Persistence

The preference is saved to localStorage and persists across sessions.

### CSS Implementation

```css
[data-theme="high-contrast"] {
    --color-bg-primary: #000000;
    --color-text-primary: #ffffff;
    --color-accent-primary: #00b4ff;
    --focus-ring: 0 0 0 4px rgba(0, 180, 255, 0.8);
}
```

---

## Color Contrast

### Minimum Requirements

All text meets WCAG 2.1 AA contrast ratios:

| Text Type | Minimum Ratio | Implementation |
|-----------|---------------|----------------|
| Normal text | 4.5:1 | `--color-text-primary` on `--color-bg-primary` |
| Large text | 3:1 | Headings and emphasis |
| UI components | 3:1 | Buttons, form controls |
| Focus indicators | 3:1 | Always visible |

### Color Variables

```css
:root {
    /* Primary text on dark background: ~12:1 */
    --color-text-primary: #e8e8f0;
    --color-bg-primary: #0f0f1a;
    
    /* Secondary text: ~6:1 */
    --color-text-secondary: #b0b0c0;
    
    /* Accent color with sufficient contrast */
    --color-accent-primary: #6366f1;
}
```

---

## Responsive Design

### Fluid Typography

All text uses relative units:

```css
html {
    font-size: 16px; /* Base size */
}

.section-title {
    font-size: var(--font-size-2xl); /* 1.5rem = 24px */
}
```

### Responsive Breakpoints

| Breakpoint | Target |
|------------|--------|
| < 640px | Mobile devices |
| 640-768px | Tablets |
| > 768px | Desktop |

### Touch Targets

All interactive elements have minimum touch target size:

```css
.btn,
.filter-input,
.filter-select {
    min-height: 44px;
    min-width: 44px;
}
```

---

## Reduced Motion

### Media Query Support

Users with motion sensitivity can enable reduced motion:

```css
@media (prefers-reduced-motion: reduce) {
    *,
    *::before,
    *::after {
        animation-duration: 0.01ms !important;
        transition-duration: 0.01ms !important;
    }
}
```

### JavaScript Detection

The application also detects system preference and adjusts:

```javascript
const prefersReducedMotion = window.matchMedia(
    '(prefers-reduced-motion: reduce)'
);
```

---

## Testing

### Automated Tests

Run the accessibility test suite:

```bash
cd frontend
node tests/a11y.test.js
```

### Manual Testing Checklist

- [ ] Navigate entire page using only keyboard
- [ ] Test with screen reader (NVDA, VoiceOver, JAWS)
- [ ] Verify all images have alt text
- [ ] Check color contrast with browser DevTools
- [ ] Test high contrast mode toggle
- [ ] Verify skip link functionality
- [ ] Test form validation announcements
- [ ] Check table keyboard navigation

### Browser DevTools

Use browser accessibility inspector:

1. Chrome: DevTools > Elements > Accessibility panel
2. Firefox: DevTools > Accessibility panel
3. Safari: Develop > Show Web Inspector > Accessibility

---

## WCAG Compliance Checklist

### Perceivable

| Criterion | Status |
|-----------|--------|
| 1.1.1 Non-text Content | ✅ All images have alt text |
| 1.2.1 Audio-only and Video-only | N/A (No media content) |
| 1.2.2 Captions (Prerecorded) | N/A |
| 1.2.3 Audio Description or Media Alternative | N/A |
| 1.3.1 Info and Relationships | ✅ Semantic HTML |
| 1.3.2 Meaningful Sequence | ✅ Correct heading hierarchy |
| 1.3.3 Sensory Characteristics | ✅ Instructions don't rely on shape/size |
| 1.3.4 Orientation | ✅ Responsive, works in both orientations |
| 1.3.5 Identify Input Purpose | ✅ Autocomplete attributes on forms |
| 1.4.1 Use of Color | ✅ Color not sole indicator |
| 1.4.2 Audio Control | N/A |
| 1.4.3 Contrast (Minimum) | ✅ All text meets 4.5:1 |
| 1.4.4 Resize Text | ✅ Uses relative units |
| 1.4.5 Images of Text | ✅ No images of text |
| 1.4.6 Contrast (Enhanced) | ✅ AAA support available |
| 1.4.7 Low or No Background Audio | N/A |
| 1.4.8 Visual Presentation | ✅ Paragraph width, spacing |
| 1.4.9 Images of Text (No Exception) | ✅ No images of text |
| 1.4.10 Reflow | ✅ Responsive without horizontal scroll |
| 1.4.11 Non-text Contrast | ✅ UI components ≥ 3:1 |
| 1.4.12 Text Spacing | ✅ Overrideable via CSS |
| 1.4.13 Content on Hover or Focus | ✅ Hover content dismissible |

### Operable

| Criterion | Status |
|-----------|--------|
| 2.1.1 Keyboard | ✅ All functionality available |
| 2.1.2 No Keyboard Trap | ✅ Escape key works |
| 2.1.3 Keyboard (No Exception) | ✅ Complete keyboard support |
| 2.1.4 Character Key Shortcuts | ✅ Modifiers available |
| 2.2.1 Timing Adjustable | ✅ No time limits |
| 2.2.2 Pause, Stop, Hide | ✅ Events can be paused |
| 2.2.3 No Timing | ✅ No time-based content |
| 2.2.4 Interruptions | ✅ User controls updates |
| 2.2.5 Re-authenticating | N/A |
| 2.2.6 Timeouts | N/A |
| 2.3.1 Three Flashes or Below Threshold | ✅ No flashing content |
| 2.3.2 Three Flashes | ✅ No flashing content |
| 2.3.3 Animation from Interactions | ✅ Reduced motion support |
| 2.4.1 Bypass Blocks | ✅ Skip link provided |
| 2.4.2 Page Titled | ✅ Descriptive page titles |
| 2.4.3 Focus Order | ✅ Logical tab order |
| 2.4.4 Link Purpose (In Context) | ✅ Clear link text |
| 2.4.5 Multiple Ways | ✅ Multiple navigation options |
| 2.4.6 Headings and Labels | ✅ Descriptive headings |
| 2.4.7 Focus Visible | ✅ Visible focus indicators |

### Understandable

| Criterion | Status |
|-----------|--------|
| 3.1.1 Language of Page | ✅ `lang="en"` |
| 3.1.2 Language of Parts | Partial |
| 3.2.1 On Focus | ✅ No unexpected changes |
| 3.2.2 On Input | ✅ No unexpected changes |
| 3.2.3 Consistent Navigation | ✅ Consistent menu |
| 3.2.4 Consistent Identification | ✅ Consistent labels |
| 3.3.1 Errors Identified | ✅ Error messages provided |
| 3.3.2 Labels or Instructions | ✅ Labels provided |
| 3.3.3 Error Suggestion | ✅ Error suggestions |
| 3.3.4 Error Prevention | ✅ Review before submit |

### Robust

| Criterion | Status |
|-----------|--------|
| 4.1.1 Parsing | ✅ Valid HTML |
| 4.1.2 Name, Role, Value | ✅ Proper ARIA usage |

---

## Support

For accessibility questions or concerns:

- Email: accessibility@stellarescrow.io
- GitHub Issues: [Accessibility Label]

---

## References

- [WCAG 2.1 Guidelines](https://www.w3.org/WAI/WCAG21/quickref/)
- [WAI-ARIA Authoring Practices](https://www.w3.org/WAI/ARIA/apg/)
- [WebAIM Contrast Checker](https://webaim.org/resources/contrastchecker/)
- [A11y Project Checklist](https://www.a11yproject.com/checklist/)
