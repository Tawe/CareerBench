/**
 * Utility functions for formatting and handling errors in the UI
 */

export interface ErrorInfo {
  message: string;
  suggestions: string[];
  recoverable: boolean;
  requiresAction: boolean;
}

/**
 * Converts raw error messages into user-friendly format with recovery suggestions.
 * Analyzes the error message to categorize the error type and provide appropriate
 * guidance to the user.
 * 
 * @param errorMessage - The raw error message from the system or API
 * @returns ErrorInfo object containing formatted message, suggestions, and metadata
 * 
 * @example
 * ```typescript
 * const errorInfo = formatErrorForUser("API key is invalid");
 * // Returns: { message: "Your API key is invalid or has expired", suggestions: [...], ... }
 * ```
 */
export function formatErrorForUser(errorMessage: string): ErrorInfo {
  const lowerMessage = errorMessage.toLowerCase();

  // Invalid API key
  if (
    lowerMessage.includes("api key") ||
    lowerMessage.includes("invalid key") ||
    lowerMessage.includes("authentication")
  ) {
    return {
      message: "Your API key is invalid or has expired",
      suggestions: [
        "Check your API key in Settings",
        "Verify the key is correct and hasn't been revoked",
        "Generate a new API key if needed",
      ],
      recoverable: false,
      requiresAction: true,
    };
  }

  // Rate limit
  if (
    lowerMessage.includes("rate limit") ||
    lowerMessage.includes("too many requests") ||
    lowerMessage.includes("429")
  ) {
    return {
      message: "Rate limit exceeded. Too many requests in a short time",
      suggestions: [
        "Wait a few moments and try again",
        "The system will automatically retry with a delay",
        "Consider upgrading your API plan for higher limits",
      ],
      recoverable: true,
      requiresAction: false,
    };
  }

  // Network errors
  if (
    lowerMessage.includes("network") ||
    lowerMessage.includes("connection") ||
    lowerMessage.includes("timeout") ||
    lowerMessage.includes("timed out") ||
    lowerMessage.includes("refused")
  ) {
    const isTimeout = lowerMessage.includes("timeout") || lowerMessage.includes("timed out");
    return {
      message: isTimeout
        ? "Connection timed out. The AI service may be slow or unavailable"
        : "Cannot connect to the AI service. Check your internet connection",
      suggestions: [
        "Check your internet connection",
        "Wait a moment and try again",
        "If the problem persists, the AI service may be temporarily unavailable",
      ],
      recoverable: true,
      requiresAction: false,
    };
  }

  // Not configured
  if (
    lowerMessage.includes("not configured") ||
    lowerMessage.includes("not set up") ||
    lowerMessage.includes("not yet implemented") ||
    lowerMessage.includes("failed to resolve provider")
  ) {
    return {
      message: "AI provider is not configured",
      suggestions: [
        "Go to Settings to configure your AI provider",
        "For cloud providers, enter your API key",
        "For local providers, specify the model file path",
      ],
      recoverable: false,
      requiresAction: true,
    };
  }

  // Model file not found
  if (
    lowerMessage.includes("model path") ||
    lowerMessage.includes("model file") ||
    lowerMessage.includes("model not found")
  ) {
    return {
      message: "Local AI model file not found",
      suggestions: [
        "Check the model path in Settings",
        "Ensure the model file exists and is accessible",
        "Download a GGUF model file if you haven't already",
      ],
      recoverable: false,
      requiresAction: true,
    };
  }

  // Invalid response
  if (
    lowerMessage.includes("invalid response") ||
    lowerMessage.includes("failed to parse") ||
    lowerMessage.includes("validation error")
  ) {
    return {
      message: "The AI service returned an unexpected response",
      suggestions: [
        "Try generating again - this is usually a temporary issue",
        "If the problem continues, the AI model may be experiencing issues",
      ],
      recoverable: true,
      requiresAction: false,
    };
  }

  // Generic error
  return {
    message: errorMessage || "An unexpected error occurred",
    suggestions: [
      "Try again in a moment",
      "If the problem persists, check your AI settings",
    ],
    recoverable: true,
    requiresAction: false,
  };
}

/**
 * Formats an ErrorInfo object into a single string for display in the UI.
 * Combines the error message with bullet-pointed suggestions.
 * 
 * @param errorInfo - The ErrorInfo object to format
 * @returns Formatted string with message and suggestions
 * 
 * @example
 * ```typescript
 * const errorInfo = formatErrorForUser("API key invalid");
 * const displayText = formatErrorWithSuggestions(errorInfo);
 * // Returns: "Your API key is invalid or has expired\n\n• Check your API key in Settings\n• ..."
 * ```
 */
export function formatErrorWithSuggestions(errorInfo: ErrorInfo): string {
  let result = errorInfo.message;
  if (errorInfo.suggestions.length > 0) {
    result += "\n\n" + errorInfo.suggestions.map((s) => `• ${s}`).join("\n");
  }
  return result;
}

