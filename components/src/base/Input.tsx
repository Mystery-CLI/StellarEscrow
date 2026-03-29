import React from 'react';
import './Input.css';

export interface InputProps extends React.InputHTMLAttributes<HTMLInputElement> {
  label?: string;
  error?: string;
  helperText?: string;
  /** Show green valid indicator */
  valid?: boolean;
}

export const Input = React.forwardRef<HTMLInputElement, InputProps>(
  ({ label, error, helperText, valid, id, className = '', ...props }, ref) => {
    const inputId = React.useId();
    const resolvedId = id || inputId;

    const stateClass = error ? 'input-error' : valid ? 'input-valid' : '';
    const describedBy = error
      ? `${resolvedId}-error`
      : helperText
      ? `${resolvedId}-helper`
      : undefined;

    return (
      <div className={`input-wrapper ${error ? 'input-wrapper--error' : valid ? 'input-wrapper--valid' : ''}`}>
        {label && (
          <label htmlFor={resolvedId} className="input-label">
            {label}
          </label>
        )}

        <div className="input-field-row">
          <input
            ref={ref}
            id={resolvedId}
            className={`input ${stateClass} ${className}`.trim()}
            aria-invalid={error ? true : undefined}
            aria-describedby={describedBy}
            {...props}
          />
          {/* Validation state icon */}
          {(error || valid) && (
            <span
              className={`input-icon ${error ? 'input-icon--error' : 'input-icon--valid'}`}
              aria-hidden="true"
            >
              {error ? '✕' : '✓'}
            </span>
          )}
        </div>

        {/* Animated error message */}
        <span
          id={`${resolvedId}-error`}
          className={`input-error-text ${error ? 'input-error-text--visible' : ''}`}
          role={error ? 'alert' : undefined}
          aria-live="polite"
        >
          {error ?? ''}
        </span>

        {helperText && !error && (
          <span id={`${resolvedId}-helper`} className="input-helper-text">
            {helperText}
          </span>
        )}
      </div>
    );
  }
);

Input.displayName = 'Input';
