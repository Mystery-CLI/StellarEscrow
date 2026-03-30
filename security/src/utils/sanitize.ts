import DOMPurify from 'dompurify';

/**
 * Sanitizes user input to prevent XSS attacks by removing script tags and unsafe HTML.
 * This is particularly important for fields like the 'SignUp description' or any rich text input.
 * 
 * @param input - The raw string input from the user.
 * @returns A sanitized string with unsafe elements removed.
 */
export const sanitizeInput = (input: string): string => {
  // We use DOMPurify for industry-standard sanitization.
  // This helps mitigate risks like <script> injection and event handler attacks (e.g., onerror).
  return DOMPurify.sanitize(input, {
    ALLOWED_TAGS: ['b', 'i', 'em', 'strong', 'a'], // Limit to safe formatting tags
    ALLOWED_ATTR: ['href', 'title'],
    KEEP_CONTENT: true,
  });
};
