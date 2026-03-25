# Component Library Development Guide

## Architecture

### Directory Structure

```
components/
├── src/
│   ├── base/              # Base UI components
│   │   ├── Button.tsx
│   │   ├── Button.css
│   │   ├── Button.stories.tsx
│   │   ├── Button.test.tsx
│   │   ├── Input.tsx
│   │   ├── Card.tsx
│   │   ├── Badge.tsx
│   │   ├── Alert.tsx
│   │   └── index.ts
│   ├── escrow/            # Escrow-specific components
│   │   ├── TradeStatus.tsx
│   │   ├── TradeCard.tsx
│   │   ├── TradeForm.tsx
│   │   ├── EventFeed.tsx
│   │   ├── *.stories.tsx
│   │   ├── *.test.tsx
│   │   └── index.ts
│   └── index.ts
├── .storybook/            # Storybook configuration
├── jest.config.js         # Jest configuration
├── tsconfig.json          # TypeScript configuration
├── package.json
└── README.md
```

## Component Development Workflow

### 1. Create Component

Create component file with TypeScript and proper typing:

```tsx
// src/base/MyComponent.tsx
import React from 'react';
import './MyComponent.css';

export interface MyComponentProps {
  // Define props
}

export const MyComponent: React.FC<MyComponentProps> = (props) => {
  return <div>{/* Component JSX */}</div>;
};
```

### 2. Add Styles

Create corresponding CSS file with scoped styles:

```css
/* src/base/MyComponent.css */
.my-component {
  /* Styles */
}
```

### 3. Create Stories

Add Storybook stories for documentation:

```tsx
// src/base/MyComponent.stories.tsx
import type { Meta, StoryObj } from '@storybook/react';
import { MyComponent } from './MyComponent';

const meta = {
  title: 'Base/MyComponent',
  component: MyComponent,
  tags: ['autodocs'],
} satisfies Meta<typeof MyComponent>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {
  args: { /* default props */ },
};
```

### 4. Add Tests

Write comprehensive tests:

```tsx
// src/base/MyComponent.test.tsx
import { render, screen } from '@testing-library/react';
import { MyComponent } from './MyComponent';

describe('MyComponent', () => {
  it('renders correctly', () => {
    render(<MyComponent />);
    // assertions
  });
});
```

### 5. Export from Index

Add to appropriate index file:

```ts
// src/base/index.ts
export { MyComponent, type MyComponentProps } from './MyComponent';
```

## Accessibility Guidelines

All components must:

1. **Semantic HTML**: Use appropriate HTML elements
2. **ARIA Labels**: Provide `aria-label` or `aria-labelledby` where needed
3. **Keyboard Navigation**: Support Tab, Enter, Escape keys
4. **Focus Management**: Visible focus indicators
5. **Color Contrast**: WCAG AA compliant (4.5:1 for text)
6. **Screen Readers**: Proper role and state announcements

## Testing Standards

- Minimum 80% code coverage
- Test user interactions, not implementation
- Use `@testing-library/react` for DOM testing
- Test accessibility features

## Storybook Best Practices

- Include all component variants
- Document props with descriptions
- Show accessibility features
- Include usage examples
- Add a11y addon checks

## Build & Deploy

```bash
# Build components
npm run build

# Build Storybook
npm run build-storybook

# Run tests
npm test

# Lint code
npm run lint
```

## Integration with Main App

Import components in your application:

```tsx
import { Button, TradeCard, TradeForm } from '@stellar-escrow/components';
```

Or import specific components:

```tsx
import { Button } from '@stellar-escrow/components/dist/base/Button';
```

## Versioning

Follow semantic versioning:
- MAJOR: Breaking changes
- MINOR: New features
- PATCH: Bug fixes

Update version in `package.json` before publishing.
