import "./LoadingSkeleton.css";

interface LoadingSkeletonProps {
  width?: string;
  height?: string;
  className?: string;
  lines?: number;
  variant?: "text" | "card" | "list" | "table";
}

export function LoadingSkeleton({
  width,
  height,
  className = "",
  lines = 1,
  variant = "text",
}: LoadingSkeletonProps) {
  if (variant === "card") {
    return (
      <div className={`skeleton-card ${className}`}>
        <div className="skeleton skeleton-title" style={{ width: width || "60%", height: height || "1.5rem" }}></div>
        <div className="skeleton skeleton-line" style={{ width: "100%", height: "1rem" }}></div>
        <div className="skeleton skeleton-line" style={{ width: "80%", height: "1rem" }}></div>
        <div className="skeleton skeleton-line" style={{ width: "90%", height: "1rem" }}></div>
      </div>
    );
  }

  if (variant === "list") {
    return (
      <div className={`skeleton-list ${className}`}>
        {Array.from({ length: lines }).map((_, i) => (
          <div key={i} className="skeleton-list-item">
            <div className="skeleton skeleton-avatar" style={{ width: "2.5rem", height: "2.5rem" }}></div>
            <div className="skeleton-list-content">
              <div className="skeleton skeleton-line" style={{ width: "60%", height: "1rem" }}></div>
              <div className="skeleton skeleton-line" style={{ width: "40%", height: "0.875rem" }}></div>
            </div>
          </div>
        ))}
      </div>
    );
  }

  if (variant === "table") {
    return (
      <div className={`skeleton-table ${className}`}>
        <div className="skeleton-table-header">
          {Array.from({ length: 5 }).map((_, i) => (
            <div key={i} className="skeleton skeleton-line" style={{ width: "100%", height: "1rem" }}></div>
          ))}
        </div>
        {Array.from({ length: lines }).map((_, i) => (
          <div key={i} className="skeleton-table-row">
            {Array.from({ length: 5 }).map((_, j) => (
              <div key={j} className="skeleton skeleton-line" style={{ width: "100%", height: "1rem" }}></div>
            ))}
          </div>
        ))}
      </div>
    );
  }

  // Default text variant
  return (
    <div className={`skeleton-text ${className}`}>
      {Array.from({ length: lines }).map((_, i) => (
        <div
          key={i}
          className="skeleton skeleton-line"
          style={{
            width: i === lines - 1 ? width || "80%" : "100%",
            height: height || "1rem",
          }}
        ></div>
      ))}
    </div>
  );
}

