/**
 * Validation utilities using Zod
 * 
 * Provides helper functions for validating data and formatting errors
 */

import { z, ZodError, ZodSchema } from 'zod';

/**
 * Validation result type
 */
export interface ValidationResult<T> {
  success: boolean;
  data?: T;
  errors?: Record<string, string>;
}

/**
 * Validates data against a Zod schema and returns a structured result
 * 
 * @param schema - Zod schema to validate against
 * @param data - Data to validate
 * @returns ValidationResult with success flag, data (if valid), and errors (if invalid)
 * 
 * @example
 * ```typescript
 * const result = validate(userProfileSchema, profileData);
 * if (result.success) {
 *   // Use result.data
 * } else {
 *   // Display result.errors
 * }
 * ```
 */
export function validate<T>(
  schema: ZodSchema<T>,
  data: unknown
): ValidationResult<T> {
  try {
    const validated = schema.parse(data);
    return {
      success: true,
      data: validated,
    };
  } catch (error) {
    if (error instanceof ZodError) {
      const errors: Record<string, string> = {};
      
      // Format Zod errors into a flat object
      error.issues.forEach((err) => {
        const path = err.path.join('.');
        if (!errors[path]) {
          errors[path] = err.message;
        }
      });
      
      return {
        success: false,
        errors,
      };
    }
    
    // Unexpected error
    return {
      success: false,
      errors: { _general: 'Validation failed' },
    };
  }
}

/**
 * Validates data and throws an error if invalid
 * 
 * @param schema - Zod schema to validate against
 * @param data - Data to validate
 * @returns Validated data
 * @throws ZodError if validation fails
 * 
 * @example
 * ```typescript
 * try {
 *   const validated = validateOrThrow(userProfileSchema, profileData);
 *   // Use validated data
 * } catch (error) {
 *   // Handle validation error
 * }
 * ```
 */
export function validateOrThrow<T>(
  schema: ZodSchema<T>,
  data: unknown
): T {
  return schema.parse(data);
}

/**
 * Safely validates data and returns null if invalid
 * 
 * @param schema - Zod schema to validate against
 * @param data - Data to validate
 * @returns Validated data or null if invalid
 * 
 * @example
 * ```typescript
 * const validated = validateSafe(userProfileSchema, profileData);
 * if (validated) {
 *   // Use validated data
 * }
 * ```
 */
export function validateSafe<T>(
  schema: ZodSchema<T>,
  data: unknown
): T | null {
  const result = schema.safeParse(data);
  return result.success ? result.data : null;
}

/**
 * Formats Zod errors into a user-friendly error message
 * 
 * @param error - ZodError instance
 * @returns Formatted error message string
 */
export function formatValidationError(error: ZodError): string {
  const messages = error.issues.map((err) => {
    const path = err.path.length > 0 ? `${err.path.join('.')}: ` : '';
    return `${path}${err.message}`;
  });
  
  return messages.join('\n');
}

/**
 * Gets the first error message from a validation result
 * 
 * @param result - ValidationResult
 * @returns First error message or undefined
 */
export function getFirstError(result: ValidationResult<unknown>): string | undefined {
  if (result.success || !result.errors) {
    return undefined;
  }
  
  const firstKey = Object.keys(result.errors)[0];
  return result.errors[firstKey];
}

/**
 * Checks if a validation result has errors for a specific field
 * 
 * @param result - ValidationResult
 * @param fieldPath - Dot-separated field path (e.g., "experience.0.company")
 * @returns Error message for the field or undefined
 */
export function getFieldError(
  result: ValidationResult<unknown>,
  fieldPath: string
): string | undefined {
  if (result.success || !result.errors) {
    return undefined;
  }
  
  return result.errors[fieldPath];
}

