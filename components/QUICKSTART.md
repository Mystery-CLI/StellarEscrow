# Component Library Quick Start

## Installation

```bash
cd components
npm install
```

## View Components in Storybook

```bash
npm run storybook
```

Opens at `http://localhost:6006`

## Run Tests

```bash
npm test
```

## Build Components

```bash
npm run build
```

Output in `dist/` directory

## Use in Your App

### Import Components

```tsx
import { 
  Button, 
  Input, 
  Card, 
  Badge,
  Alert,
  TradeCard,
  TradeForm,
  TradeStatus,
  EventFeed
} from '@stellar-escrow/components';
```

### Example: Create Trade Form

```tsx
import { TradeForm } from '@stellar-escrow/components';

function CreateTrade() {
  const handleSubmit = (data) => {
    console.log('Creating trade:', data);
    // Call API to create trade
  };

  return (
    <TradeForm 
      onSubmit={handleSubmit}
      loading={false}
    />
  );
}
```

### Example: Display Trade

```tsx
import { TradeCard } from '@stellar-escrow/components';

function TradeList() {
  return (
    <TradeCard
      tradeId="12345"
      seller="GBUQWP3BOUZX34ULNQG23RQ6F4BVWCIYU2IYJJMTEN4D7NOXVJPPJNBE"
      buyer="GBBD47UZQ5DYWVV4YPVYZKRYE7JQ63ERCXZLP4GDQFVRJQG5FDORBDD"
      amount="100.50"
      status="completed"
      timestamp="2024-03-25 10:30:00"
      onClick={() => console.log('Trade clicked')}
    />
  );
}
```

### Example: Event Feed

```tsx
import { EventFeed } from '@stellar-escrow/components';

function EventLog() {
  const events = [
    {
      id: '1',
      type: 'trade_created',
      tradeId: '12345',
      timestamp: '2024-03-25 10:30:00',
      data: {},
    },
    {
      id: '2',
      type: 'trade_funded',
      tradeId: '12345',
      timestamp: '2024-03-25 10:35:00',
      data: {},
    },
  ];

  return (
    <EventFeed 
      events={events}
      onEventClick={(event) => console.log('Event clicked:', event)}
    />
  );
}
```

## Component Reference

### Base Components

- **Button** - Interactive button with variants
- **Input** - Form input with validation
- **Card** - Content container
- **Badge** - Status indicator
- **Alert** - Notification message

### Escrow Components

- **TradeStatus** - Trade state badge
- **TradeCard** - Trade information display
- **TradeForm** - Trade creation form
- **EventFeed** - Real-time event display

## Documentation

- [README.md](./README.md) - Full API documentation
- [DEVELOPMENT.md](./DEVELOPMENT.md) - Development guidelines
- [ACCEPTANCE_CRITERIA.md](./ACCEPTANCE_CRITERIA.md) - Implementation checklist

## Storybook Stories

View interactive examples of all components:

```bash
npm run storybook
```

## Testing

Run all tests:

```bash
npm test
```

Watch mode:

```bash
npm run test:watch
```

## Build for Production

```bash
npm run build
```

Creates optimized `dist/` directory ready for npm publishing.

## Next Steps

1. ✅ Install dependencies: `npm install`
2. ✅ View Storybook: `npm run storybook`
3. ✅ Run tests: `npm test`
4. ✅ Build: `npm run build`
5. ✅ Integrate into main app
6. ✅ Publish to npm (optional)

## Support

For issues or questions:
- Check [README.md](./README.md) for API documentation
- Review [DEVELOPMENT.md](./DEVELOPMENT.md) for guidelines
- View Storybook stories for examples
- Check test files for usage patterns
