export const seedTrades = [
  {
    id: '1',
    seller: 'GBUQWP3BOUZX34ULNQG23RQ6F4BVWCIYU2IYJJMTEN4D7NOXVJPPJNBE',
    buyer: 'GBBD47UZQ5DYWVV4YPVYZKRYE7JQ63ERCXZLP4GDQFVRJQG5FDORBDD',
    amount: '100.50',
    status: 'completed',
    timestamp: '2024-03-25T10:30:00Z',
  },
];

export const seedEvents = [
  {
    id: '1',
    type: 'trade_created',
    category: 'trade',
    schemaVersion: 1,
    tradeId: '1',
    timestamp: '2024-03-25T10:30:00Z',
    data: {},
  },
];

export const mockTrades = [...seedTrades];
export const mockEvents = [...seedEvents];

export function resetMockData() {
  mockTrades.length = 0;
  mockTrades.push(...seedTrades);
  mockEvents.length = 0;
  mockEvents.push(...seedEvents);
}
