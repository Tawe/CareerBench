import React from 'react';
import './ProgressIndicator.css';

export type ProgressStep = {
  id: string;
  label: string;
  status: 'pending' | 'active' | 'completed' | 'error';
};

interface ProgressIndicatorProps {
  currentStep?: string;
  steps?: ProgressStep[];
  message?: string;
  progress?: number; // 0-100 for progress bar
  variant?: 'spinner' | 'steps' | 'bar' | 'compact';
}

export function ProgressIndicator({
  currentStep,
  steps,
  message,
  progress,
  variant = 'spinner',
}: ProgressIndicatorProps) {
  if (variant === 'compact') {
    return (
      <div className="progress-indicator compact">
        <div className="progress-spinner-small"></div>
        {message && <span className="progress-message-small">{message}</span>}
      </div>
    );
  }

  if (variant === 'bar' && progress !== undefined) {
    return (
      <div className="progress-indicator bar">
        {message && <div className="progress-message">{message}</div>}
        <div className="progress-bar-container">
          <div 
            className="progress-bar-fill" 
            style={{ width: `${Math.min(100, Math.max(0, progress))}%` }}
          ></div>
        </div>
        {progress < 100 && (
          <div className="progress-percentage">{Math.round(progress)}%</div>
        )}
      </div>
    );
  }

  if (variant === 'steps' && steps) {
    const currentIndex = steps.findIndex(s => s.id === currentStep || s.status === 'active');
    
    return (
      <div className="progress-indicator steps">
        {message && <div className="progress-message">{message}</div>}
        <div className="progress-steps">
          {steps.map((step, index) => (
            <div
              key={step.id}
              className={`progress-step ${step.status} ${index === currentIndex ? 'current' : ''}`}
            >
              <div className="progress-step-indicator">
                {step.status === 'completed' && (
                  <svg width="16" height="16" viewBox="0 0 16 16" fill="none">
                    <path
                      d="M13.5 4.5L6 12L2.5 8.5"
                      stroke="currentColor"
                      strokeWidth="2"
                      strokeLinecap="round"
                      strokeLinejoin="round"
                    />
                  </svg>
                )}
                {step.status === 'active' && <div className="progress-step-spinner"></div>}
                {step.status === 'error' && (
                  <svg width="16" height="16" viewBox="0 0 16 16" fill="none">
                    <path
                      d="M4 4L12 12M12 4L4 12"
                      stroke="currentColor"
                      strokeWidth="2"
                      strokeLinecap="round"
                    />
                  </svg>
                )}
                {step.status === 'pending' && <div className="progress-step-dot"></div>}
              </div>
              <div className="progress-step-label">{step.label}</div>
              {index < steps.length - 1 && (
                <div className={`progress-step-connector ${step.status === 'completed' ? 'completed' : ''}`}></div>
              )}
            </div>
          ))}
        </div>
      </div>
    );
  }

  // Default spinner variant
  return (
    <div className="progress-indicator spinner">
      {message && <div className="progress-message">{message}</div>}
      <div className="progress-spinner"></div>
    </div>
  );
}

