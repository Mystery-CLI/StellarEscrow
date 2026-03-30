import { friendlyMessage } from '../../ErrorBoundary';

describe('friendlyMessage', () => {
  it('returns mapped message for known error code', () => {
    const error = Object.assign(new Error('raw error'), { code: 404 });

    expect(friendlyMessage(error)).toBe('The requested resource was not found.');
  });

  it('returns default message when error is null', () => {
    expect(friendlyMessage(null)).toBe('An unexpected error occurred. Please try again.');
  });
});
