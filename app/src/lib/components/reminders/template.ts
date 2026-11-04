// Pure message-template rendering. No imports: it runs under `node --test` as well as in the app.

export const PLACEHOLDERS = ['sahip', 'hasta', 'asi', 'tarih', 'saat', 'tutar', 'klinik', 'telefon'] as const;
export type Placeholder = (typeof PLACEHOLDERS)[number];
export type TemplateValues = Partial<Record<Placeholder, string | null | undefined>>;

const TOKEN = /\{([^{}\s]+)\}/g;

function isPlaceholder(name: string): name is Placeholder {
  return (PLACEHOLDERS as readonly string[]).includes(name);
}

/**
 * Replaces `{name}` tokens with their values in a single pass (a value that itself contains
 * braces is never expanded again). Known placeholders without a value become empty and the
 * resulting gaps are tidied; unknown tokens are kept as typed so mistakes stay visible.
 */
export function renderTemplate(template: string, values: TemplateValues): string {
  const filled = template.replace(TOKEN, (token, name: string) => (isPlaceholder(name) ? (values[name] ?? '').trim() : token));
  return filled
    .replace(/\r\n?/g, '\n')
    .split('\n')
    .map(line => line.replace(/[ \t]{2,}/g, ' ').replace(/\s+([,.!?;:])/g, '$1').trim())
    .join('\n')
    .replace(/\n{3,}/g, '\n\n')
    .trim();
}

/** Tokens in the template that are not one of the supported placeholders. */
export function unknownPlaceholders(template: string): string[] {
  const found = new Set<string>();
  for (const match of template.matchAll(TOKEN)) if (!isPlaceholder(match[1])) found.add(match[0]);
  return [...found];
}
