import { resetMockData } from '../mocks';
import { server } from '../mocks/server';

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
