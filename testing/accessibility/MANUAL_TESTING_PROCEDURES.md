# Accessibility Manual Testing Procedures

## Overview

Automated tools catch ~30-40% of accessibility issues. This document covers
the manual procedures required to validate the remaining issues, particularly
screen reader compatibility and cognitive accessibility.

---

## 1. Keyboard-Only Navigation Checklist

Perform with mouse disconnected or disabled.

### Setup
- Open the application in Chrome or Firefox
- Disconnect or disable the mouse
- Start from the browser address bar

### Test Steps

| Step | Action | Expected Result | Pass/Fail |
|------|--------|-----------------|-----------|
| 1 | Press Tab from address bar | Focus moves to skip link | |
| 2 | Press Enter on skip link | Focus jumps to `#main-content` | |
| 3 | Tab through all nav items | Each item receives visible focus | |
| 4 | Press Enter on "Dashboard" nav link | Page navigates correctly | |
| 5 | Tab to "Create Trade" button | Button receives visible focus | |
| 6 | Press Enter on "Create Trade" | Trade form opens | |
| 7 | Tab through all form fields | Each field receives focus in logical order | |
| 8 | Fill form using keyboard only | All fields are fillable | |
| 9 | Press Tab past last field | Focus moves to Submit button | |
| 10 | Press Enter to submit | Form submits or shows validation errors | |
| 11 | Tab to a dropdown | Dropdown receives focus | |
| 12 | Press Space/Enter to open dropdown | Dropdown opens | |
| 13 | Press Arrow keys in dropdown | Options are navigable | |
| 14 | Press Escape | Dropdown closes, focus returns to trigger | |
| 15 | Tab to any modal trigger | Trigger receives focus | |
| 16 | Open modal | Focus moves inside modal | |
| 17 | Tab through modal | Focus stays within modal (focus trap) | |
| 18 | Press Escape | Modal closes, focus returns to trigger | |

### Pass Criteria
- All interactive elements reachable by Tab
- No keyboard traps (except intentional modal focus trap)
- All actions completable without mouse
- Focus indicator visible at all times

---

## 2. Screen Reader Testing Procedures

### 2.1 NVDA (Windows — Free)

**Setup:**
1. Download NVDA from nvaccess.org
2. Install and launch
3. Open Chrome or Firefox

**Test Script:**
```
1. Press NVDA+F7 to open Elements List
   → Verify: landmarks listed (banner, navigation, main, contentinfo)

2. Press H to navigate by headings
   → Verify: h1 "StellarEscrow Dashboard" announced first
   → Verify: h2 sections follow in logical order

3. Press F to navigate by form fields
   → Verify: each field announced with label + type
   → Verify: required fields announced as "required"

4. Navigate to trade form, fill seller address with invalid value
   → Verify: error message announced after blur

5. Navigate to event feed
   → Verify: new events announced via aria-live region

6. Press NVDA+Space on high contrast toggle
   → Verify: "pressed" state announced

7. Navigate to a data table
   → Verify: column headers announced with each cell
```

### 2.2 VoiceOver (macOS — Built-in)

**Setup:**
1. Press Cmd+F5 to enable VoiceOver
2. Open Safari (best VoiceOver support)

**Test Script:**
```
1. Press VO+U to open Rotor
   → Verify: Landmarks, Headings, Links, Form Controls listed

2. Navigate to Headings in Rotor
   → Verify: heading hierarchy is logical

3. Press VO+Shift+Down to interact with form
   → Verify: labels read before inputs

4. Tab to a button with only an icon
   → Verify: aria-label is announced (not just the icon character)

5. Trigger a toast notification
   → Verify: notification announced without interrupting current reading

6. Navigate to trade status badge
   → Verify: status text announced (not just color)
```

### 2.3 JAWS (Windows — Commercial)

**Setup:**
1. Launch JAWS
2. Open Chrome

**Test Script:**
```
1. Press Insert+F6 for Headings List
   → Verify: all headings present and in order

2. Press Insert+F7 for Links List
   → Verify: all links have descriptive text (no "click here")

3. Press Insert+F5 for Form Fields List
   → Verify: all fields have labels

4. Navigate to error state
   → Verify: aria-invalid and error message announced

5. Test virtual cursor vs. application mode
   → Verify: custom widgets switch to application mode correctly
```

---

## 3. Color and Contrast Verification

### Tools Required
- Chrome DevTools (built-in)
- WebAIM Contrast Checker: https://webaim.org/resources/contrastchecker/
- Colour Contrast Analyser (desktop app)

### Test Steps

| Element | Foreground | Background | Required Ratio | Actual Ratio | Pass/Fail |
|---------|-----------|-----------|----------------|--------------|-----------|
| Body text | `#e8e8f0` | `#0f0f1a` | 4.5:1 | | |
| Secondary text | `#b0b0c0` | `#0f0f1a` | 4.5:1 | | |
| Button text | `#ffffff` | `#6366f1` | 4.5:1 | | |
| Link text | `#818cf8` | `#0f0f1a` | 4.5:1 | | |
| Error text | `#f87171` | `#0f0f1a` | 4.5:1 | | |
| Focus ring | `#6366f1` | `#0f0f1a` | 3:1 (UI) | | |
| High contrast text | `#ffffff` | `#000000` | 4.5:1 | | |

### High Contrast Mode Test
1. Activate high contrast mode (button or Alt+H)
2. Verify all text remains readable
3. Verify focus indicators are visible
4. Verify no information is conveyed by color alone

---

## 4. Zoom and Reflow Testing (WCAG 1.4.4, 1.4.10)

### Test Steps
1. Set browser zoom to 200%
   - Verify: no horizontal scrollbar appears
   - Verify: all content still readable
   - Verify: no text truncated or overlapping

2. Set browser zoom to 400%
   - Verify: content reflows to single column
   - Verify: all functionality still accessible

3. Set OS text size to "Large"
   - Verify: layout adapts without breaking

---

## 5. Reduced Motion Testing (WCAG 2.3.3)

### Test Steps
1. Enable "Reduce Motion" in OS settings
   - macOS: System Preferences > Accessibility > Display > Reduce Motion
   - Windows: Settings > Ease of Access > Display > Show animations
2. Reload the application
3. Verify: animations are disabled or significantly reduced
4. Verify: transitions are instant or very short (<100ms)

---

## 6. Sign-Off

| Tester | Date | Environment | Screen Reader | Result |
|--------|------|-------------|---------------|--------|
| | | | NVDA | |
| | | | VoiceOver | |
| | | | JAWS | |
| | | Keyboard only | N/A | |
| | | 200% zoom | N/A | |
| | | Reduced motion | N/A | |
