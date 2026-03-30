# StellarEscrow Component Library

A production-ready, accessible React component library for the StellarEscrow platform.

## Overview

This library provides reusable UI components specifically designed for the escrow platform, including base components and escrow-specific components with full TypeScript support, Storybook documentation, and comprehensive tests.

## Installation

```bash
npm install @stellar-escrow/components
```

## Components

### Base Components

#### Button
Primary interactive element with multiple variants and sizes.

```tsx
import { Button } from '@stellar-escrow/components';

<Button variant="primary" size="md" onClick={handleClick}>
  Click me
</Button>
```

**Props:**
- `variant`: 'primary' | 'secondary' | 'danger' | 'success' (default: 'primary')
- `size`: 'sm' | 'md' | 'lg' (default: 'md')
- `loading`: boolean (default: false)
- `disabled`: boolean (default: false)

#### Input
Form input with label, error handling, and helper text.

```tsx
import { Input } from '@stellar-escrow/components';

<Input
  label="Email"
  type="email"
  error={errors.email}
  helperText="We'll never share your email"
  onChange={handleChange}
/>
```

**Props:**
- `label`: string (optional)
- `error`: string (optional)
- `helperText`: string (optional)
- All standard HTML input attributes

#### Card
Container component for grouping related content.

```tsx
import { Card } from '@stellar-escrow/components';

<Card title="Trade Details">
  <p>Trade information goes here</p>
</Card>
```

**Props:**
- `title`: string (optional)
- `children`: React.ReactNode

#### Badge
Small label component for status indicators.

```tsx
import { Badge } from '@stellar-escrow/components';

<Badge variant="success">Completed</Badge>
```

**Props:**
- `variant`: 'default' | 'success' | 'warning' | 'danger' | 'info' (default: 'default')

#### Alert
Notification component for messages and alerts.

```tsx
import { Alert } from '@stellar-escrow/components';

<Alert type="success" title="Success" onClose={handleClose}>
  Trade completed successfully
</Alert>
```

**Props:**
- `type`: 'success' | 'error' | 'warning' | 'info' (default: 'info')
- `title`: string (optional)
- `onClose`: () => void (optional)

#### Avatar
User profile picture or placeholder with initials.

```tsx
import { Avatar } from '@stellar-escrow/components';

<Avatar
  src="https://example.com/user.jpg"
  alt="User Name"
  size="md"
  variant="circle"
  fallback="User Name"
/>
```

**Props:**
- `src`: string (optional)
- `alt`: string (optional)
- `size`: 'sm' | 'md' | 'lg' | 'xl' (default: 'md')
- `variant`: 'circle' | 'square' (default: 'circle')
- `fallback`: string (initials will be derived from this if src fails or is missing)

### Escrow Components

#### TradeStatus
Status badge specific to trade states.

```tsx
import { TradeStatus } from '@stellar-escrow/components';

<TradeStatus status="completed" />
```

**Props:**
- `status`: 'created' | 'funded' | 'completed' | 'disputed' | 'cancelled'

#### TradeCard
Display card for individual trades with all key information.

```tsx
import { TradeCard } from '@stellar-escrow/components';

<TradeCard
  tradeId="12345"
  seller="GBUQWP3BOUZX34ULNQG23RQ6F4BVWCIYU2IYJJMTEN4D7NOXVJPPJNBE"
  buyer="GBBD47UZQ5DYWVV4YPVYZKRYE7JQ63ERCXZLP4GDQFVRJQG5FDORBDD"
  amount="100.50"
  status="completed"
  timestamp="2024-03-25 10:30:00"
  onClick={handleTradeClick}
/>
```

**Props:**
- `tradeId`: string
- `seller`: string (Stellar address)
- `buyer`: string (Stellar address)
- `amount`: string (USDC amount)
- `status`: 'created' | 'funded' | 'completed' | 'disputed' | 'cancelled'
- `timestamp`: string
- `onClick`: () => void (optional)

#### TradeForm
Form for creating new trades with validation.

```tsx
import { TradeForm } from '@stellar-escrow/components';

<TradeForm
  onSubmit={(data) => createTrade(data)}
  loading={isSubmitting}
/>
```

**Props:**
- `onSubmit`: (data: TradeFormData) => void
- `loading`: boolean (default: false)

**TradeFormData:**
```typescript
{
  seller: string;
  buyer: string;
  amount: string;
  arbitrator?: string;
}
```

#### EventFeed
Real-time event feed display with filtering.

```tsx
import { EventFeed } from '@stellar-escrow/components';

<EventFeed
  events={events}
  onEventClick={handleEventClick}
/>
```

**Props:**
- `events`: Event[]
- `onEventClick`: (event: Event) => void (optional)

**Event:**
```typescript
{
  id: string;
  type: string;
  tradeId: string;
  timestamp: string;
  data: Record<string, any>;
}
```

## Storybook

View interactive component documentation:

```bash
npm run storybook
```

Opens at `http://localhost:6006`

## Testing

Run component tests:

```bash
npm test
```

Watch mode:

```bash
npm run test:watch
```

## Accessibility

All components follow WCAG 2.1 AA standards:
- Proper semantic HTML
- ARIA labels and descriptions
- Keyboard navigation support
- Focus management
- Color contrast compliance
- Screen reader friendly

## Styling

Components use CSS modules for scoped styling. Customize by overriding CSS variables or importing component styles:

```tsx
import '@stellar-escrow/components/dist/base/Button.css';
```

## TypeScript

Full TypeScript support with exported types:

```tsx
import { Button, ButtonProps } from '@stellar-escrow/components';

const MyButton: React.FC<ButtonProps> = (props) => {
  return <Button {...props} />;
};
```

## Contributing

1. Create components in `src/base` or `src/escrow`
2. Add Storybook stories (`.stories.tsx`)
3. Add tests (`.test.tsx`)
4. Update documentation
5. Run tests: `npm test`
6. Build: `npm run build`

## License

MIT
