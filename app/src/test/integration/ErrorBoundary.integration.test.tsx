import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { ErrorBoundary } from '../../ErrorBoundary';

function CrashComponent() {
  throw Object.assign(new Error('Not found'), { code: 404 });
}

function HealthyComponent() {
  return <div>Healthy content</div>;
}

describe('ErrorBoundary integration', () => {
  it('renders fallback message and recovers on retry', async () => {
    const user = userEvent.setup();
    const consoleErrorSpy = jest.spyOn(console, 'error').mockImplementation(() => {});

    const { rerender } = render(
      <ErrorBoundary>
        <CrashComponent />
      </ErrorBoundary>
    );

    expect(screen.getByRole('alert')).toBeInTheDocument();
    expect(screen.getByText('The requested resource was not found.')).toBeInTheDocument();

    await user.click(screen.getByRole('button', { name: /try again/i }));

    rerender(
      <ErrorBoundary>
        <HealthyComponent />
      </ErrorBoundary>
    );

    expect(screen.getByText('Healthy content')).toBeInTheDocument();
    consoleErrorSpy.mockRestore();
  });
});
