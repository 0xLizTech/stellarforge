/**
 * SDK Deprecation Policy and Helper Utilities (#198).
 */
export function warnDeprecated(methodName: string, alternative: string, removalVersion = "1.0.0"): void {
  if (typeof console !== "undefined" && console.warn) {
    console.warn(
      `[DEPRECATION WARNING]: "${methodName}" is deprecated and will be removed in v${removalVersion}. Use "${alternative}" instead.`
    );
  }
}
