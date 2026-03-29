import { resetMockData, server } from '../mocks';

beforeAll(() => {
  server.listen({ onUnhandledRequest: 'error' });
});

afterEach(() => {
  server.resetHandlers();
  resetMockData();
  jest.clearAllMocks();
});

afterAll(() => {
  server.close();
});
