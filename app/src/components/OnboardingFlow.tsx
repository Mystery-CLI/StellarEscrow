import { useEffect, useMemo, useState } from 'react';
import { Link } from 'react-router-dom';

type StepStatus = 'pending' | 'completed' | 'skipped';

interface OnboardingStep {
  id: string;
  title: string;
  description: string;
  tutorial: string[];
  actionLabel: string;
  actionTo: string;
}

const STORAGE_KEY = 'stellarEscrow.onboarding.v1';

const ONBOARDING_STEPS: OnboardingStep[] = [
  {
    id: 'register-profile',
    title: 'Connect Your Wallet',
    description: 'Connect a Stellar wallet so the app can identify your account and load your trades.',
    tutorial: [
      'Use the wallet button in the top navigation.',
      'Authorize access from your wallet provider.',
      'Confirm the connected address is the one you plan to trade with.',
    ],
    actionLabel: 'Go to Dashboard',
    actionTo: '/',
  },
  {
    id: 'acknowledge-fees',
    title: 'Review Escrow Fees',
    description: 'Understand how fees are applied before creating or funding a trade.',
    tutorial: [
      'Open the create trade screen and review amount inputs.',
      'Check total trade amount and expected network/escrow costs.',
      'Proceed only after fee expectations are clear for both parties.',
    ],
    actionLabel: 'Open New Trade Form',
    actionTo: '/trades/new',
  },
  {
    id: 'create-template',
    title: 'Prepare Trade Setup',
    description: 'Fill out seller, buyer, amount, and arbitrator details as your setup wizard step.',
    tutorial: [
      'Enter seller and buyer addresses carefully.',
      'Set trade amount and select an arbitrator.',
      'Use form validation feedback before submitting.',
    ],
    actionLabel: 'Continue Setup Wizard',
    actionTo: '/trades/new',
  },
  {
    id: 'create-first-trade',
    title: 'Create Your First Trade',
    description: 'Submit your first escrow trade and verify it appears in the trade dashboard.',
    tutorial: [
      'Submit the form after reviewing trade details.',
      'Open the trade details page to verify status and events.',
      'Share trade ID with your counterparty for funding and settlement.',
    ],
    actionLabel: 'Create Trade Now',
    actionTo: '/trades/new',
  },
];

function isStepStatus(value: unknown): value is StepStatus {
  return value === 'pending' || value === 'completed' || value === 'skipped';
}

function defaultStatuses(): StepStatus[] {
  return ONBOARDING_STEPS.map(() => 'pending');
}

function loadStatuses(): StepStatus[] {
  if (typeof window === 'undefined') {
    return defaultStatuses();
  }

  try {
    const raw = window.localStorage.getItem(STORAGE_KEY);
    if (!raw) {
      return defaultStatuses();
    }

    const parsed = JSON.parse(raw);
    if (!Array.isArray(parsed) || parsed.length !== ONBOARDING_STEPS.length) {
      return defaultStatuses();
    }

    if (parsed.every(isStepStatus)) {
      return parsed;
    }

    return defaultStatuses();
  } catch {
    return defaultStatuses();
  }
}

function saveStatuses(statuses: StepStatus[]): void {
  try {
    window.localStorage.setItem(STORAGE_KEY, JSON.stringify(statuses));
  } catch {
    // Ignore storage failures in restricted browser modes.
  }
}

function findNextPending(statuses: StepStatus[]): number {
  return statuses.findIndex((status) => status === 'pending');
}

function formatStatus(status: StepStatus): string {
  if (status === 'completed') {
    return 'Completed';
  }

  if (status === 'skipped') {
    return 'Skipped';
  }

  return 'Pending';
}

export default function OnboardingFlow() {
  const [stepStatuses, setStepStatuses] = useState<StepStatus[]>(() => loadStatuses());
  const [activeStepIndex, setActiveStepIndex] = useState<number>(0);

  useEffect(() => {
    saveStatuses(stepStatuses);
  }, [stepStatuses]);

  useEffect(() => {
    if (stepStatuses[activeStepIndex] === 'pending') {
      return;
    }

    const nextPending = findNextPending(stepStatuses);
    if (nextPending >= 0) {
      setActiveStepIndex(nextPending);
    }
  }, [activeStepIndex, stepStatuses]);

  const pendingCount = useMemo(
    () => stepStatuses.filter((status) => status === 'pending').length,
    [stepStatuses]
  );
  const completedCount = useMemo(
    () => stepStatuses.filter((status) => status === 'completed').length,
    [stepStatuses]
  );
  const skippedCount = useMemo(
    () => stepStatuses.filter((status) => status === 'skipped').length,
    [stepStatuses]
  );

  const resolvedCount = ONBOARDING_STEPS.length - pendingCount;
  const progressPercent = Math.round((resolvedCount / ONBOARDING_STEPS.length) * 100);
  const hasPendingSteps = pendingCount > 0;

  const currentStep = ONBOARDING_STEPS[activeStepIndex];
  const currentStatus = stepStatuses[activeStepIndex];

  const markCurrentStep = (status: Extract<StepStatus, 'completed' | 'skipped'>) => {
    setStepStatuses((previous) => {
      if (previous[activeStepIndex] !== 'pending') {
        return previous;
      }

      const next = [...previous];
      next[activeStepIndex] = status;
      return next;
    });
  };

  const skipAll = () => {
    setStepStatuses((previous) => previous.map((status) => (status === 'pending' ? 'skipped' : status)));
  };

  const restartOnboarding = () => {
    setStepStatuses(defaultStatuses());
    setActiveStepIndex(0);
  };

  if (!hasPendingSteps) {
    return (
      <section className="onboarding" aria-labelledby="onboarding-title">
        <div className="onboarding-header">
          <h2 id="onboarding-title">Onboarding Complete</h2>
          <p>
            Completed {completedCount} of {ONBOARDING_STEPS.length} steps.
            {skippedCount > 0 ? ` Skipped ${skippedCount} step(s).` : ''}
          </p>
        </div>
        <div className="onboarding-summary-actions">
          <button type="button" className="onboarding-btn secondary" onClick={restartOnboarding}>
            Restart Onboarding
          </button>
        </div>
      </section>
    );
  }

  return (
    <section className="onboarding" aria-labelledby="onboarding-title">
      <div className="onboarding-header">
        <h2 id="onboarding-title">Getting Started</h2>
        <p>
          {resolvedCount} of {ONBOARDING_STEPS.length} steps complete ({progressPercent}%).
        </p>
        <div className="onboarding-progress" aria-label="Onboarding progress">
          <div className="onboarding-progress-bar" style={{ width: `${progressPercent}%` }} />
        </div>
      </div>

      <ol className="onboarding-steps" aria-label="Onboarding sequence">
        {ONBOARDING_STEPS.map((step, index) => {
          const status = stepStatuses[index];
          const active = activeStepIndex === index;

          return (
            <li key={step.id}>
              <button
                type="button"
                className={`onboarding-step ${active ? 'active' : ''}`}
                onClick={() => setActiveStepIndex(index)}
                aria-current={active ? 'step' : undefined}
              >
                <span className="onboarding-step-index">{index + 1}</span>
                <span className="onboarding-step-content">
                  <span className="onboarding-step-title">{step.title}</span>
                  <span className={`onboarding-step-status ${status}`}>{formatStatus(status)}</span>
                </span>
              </button>
            </li>
          );
        })}
      </ol>

      <article className="onboarding-card" aria-live="polite">
        <h3>{currentStep.title}</h3>
        <p>{currentStep.description}</p>

        <div className="onboarding-tutorial">
          <h4>Tutorial</h4>
          <ul>
            {currentStep.tutorial.map((item) => (
              <li key={item}>{item}</li>
            ))}
          </ul>
          <Link to={currentStep.actionTo} className="onboarding-link">
            {currentStep.actionLabel}
          </Link>
        </div>

        <div className="onboarding-controls">
          <button
            type="button"
            className="onboarding-btn secondary"
            onClick={() => setActiveStepIndex((current) => Math.max(0, current - 1))}
            disabled={activeStepIndex === 0}
          >
            Previous Step
          </button>
          <button
            type="button"
            className="onboarding-btn secondary"
            onClick={() =>
              setActiveStepIndex((current) => Math.min(ONBOARDING_STEPS.length - 1, current + 1))
            }
            disabled={activeStepIndex === ONBOARDING_STEPS.length - 1}
          >
            Next Step
          </button>
        </div>

        <div className="onboarding-actions">
          <button
            type="button"
            className="onboarding-btn primary"
            onClick={() => markCurrentStep('completed')}
            disabled={currentStatus !== 'pending'}
          >
            Mark Complete
          </button>
          <button
            type="button"
            className="onboarding-btn secondary"
            onClick={() => markCurrentStep('skipped')}
            disabled={currentStatus !== 'pending'}
          >
            Skip Step
          </button>
          <button type="button" className="onboarding-btn danger" onClick={skipAll}>
            Skip Onboarding
          </button>
        </div>
      </article>
    </section>
  );
}
