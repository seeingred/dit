interface CommitOverlayProps {
  steps: string[];
  isComplete: boolean;
}

export function CommitOverlay({ steps, isComplete }: CommitOverlayProps) {
  return (
    <div className="fixed inset-0 bg-black/50 backdrop-blur-sm flex items-center justify-center z-50">
      <div className="bg-dit-surface border border-dit-border rounded-xl p-6 w-96 shadow-2xl">
        <div className="flex items-center gap-3 mb-5">
          {!isComplete ? (
            <svg
              className="w-5 h-5 animate-spin text-dit-accent flex-shrink-0"
              viewBox="0 0 24 24"
              fill="none"
            >
              <circle
                className="opacity-25"
                cx="12"
                cy="12"
                r="10"
                stroke="currentColor"
                strokeWidth="4"
              />
              <path
                className="opacity-75"
                fill="currentColor"
                d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"
              />
            </svg>
          ) : (
            <svg
              className="w-5 h-5 text-dit-success flex-shrink-0"
              fill="none"
              viewBox="0 0 24 24"
              stroke="currentColor"
            >
              <path
                strokeLinecap="round"
                strokeLinejoin="round"
                strokeWidth={2}
                d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z"
              />
            </svg>
          )}
          <h3 className="text-lg font-semibold text-dit-text">
            {isComplete ? "Commit Complete" : "Committing..."}
          </h3>
        </div>

        <div className="space-y-2.5">
          {steps.map((step, i) => {
            const isDone = i < steps.length - 1 || isComplete;
            const isCurrent = i === steps.length - 1 && !isComplete;

            return (
              <div key={i} className="flex items-start gap-2.5">
                {isDone ? (
                  <svg
                    className="w-4 h-4 text-dit-success flex-shrink-0 mt-0.5"
                    fill="none"
                    viewBox="0 0 24 24"
                    stroke="currentColor"
                  >
                    <path
                      strokeLinecap="round"
                      strokeLinejoin="round"
                      strokeWidth={2}
                      d="M5 13l4 4L19 7"
                    />
                  </svg>
                ) : isCurrent ? (
                  <div className="w-4 h-4 flex-shrink-0 mt-0.5 flex items-center justify-center">
                    <div className="w-2 h-2 bg-dit-accent rounded-full animate-pulse" />
                  </div>
                ) : (
                  <div className="w-4 h-4 flex-shrink-0 mt-0.5" />
                )}
                <span
                  className={`text-sm leading-snug ${
                    isDone
                      ? "text-dit-text-muted"
                      : isCurrent
                        ? "text-dit-text font-medium"
                        : "text-dit-text-muted/50"
                  }`}
                >
                  {step}
                </span>
              </div>
            );
          })}
        </div>
      </div>
    </div>
  );
}
