import type { ApiError, VisitTemplate, NewVisitTemplate, VisitTemplatePatch } from './types';
interface Context { now(): string; fail(kind: ApiError['kind'], message: string, field?: string): never }
export function createTemplatesMock(ctx: Context) {
  let next = 5000;
  // Start empty in the browser too: templates always belong to the veterinarian.
  const rows: VisitTemplate[] = [];
  const get = (id: number) => rows.find(r => r.id === id) ?? ctx.fail('not_found', 'Record not found.');
  const clean = (value: string | null | undefined) => value?.trim() || null;
  function validate(input: NewVisitTemplate) { if (!input.name?.trim()) ctx.fail('invalid', 'Must not be empty.', 'name'); }
  const handlers: Record<string, (args: unknown) => unknown> = {
    visit_templates_list: args => rows.filter(r => (args as { includeDeleted: boolean }).includeDeleted || !r.deletedAt).sort((a, b) => a.name.localeCompare(b.name)),
    visit_template_get: args => get((args as { id: number }).id),
    visit_template_create: args => {
      const { input } = args as { input: NewVisitTemplate }; validate(input); const stamp = ctx.now();
      const row: VisitTemplate = { id: next++, name: input.name.trim(), reason: clean(input.reason), findings: clean(input.findings), diagnosis: clean(input.diagnosis), treatment: clean(input.treatment), notes: clean(input.notes), createdAt: stamp, updatedAt: stamp, deletedAt: null };
      rows.push(row); return row;
    },
    visit_template_update: args => {
      const { id, patch } = args as { id: number; patch: VisitTemplatePatch }; const row = get(id);
      if (row.deletedAt) ctx.fail('not_found', 'Record not found.'); const merged = { ...row, ...patch }; validate(merged);
      for (const key of ['reason', 'findings', 'diagnosis', 'treatment', 'notes'] as const) merged[key] = clean(merged[key]);
      Object.assign(row, merged, { name: merged.name.trim(), updatedAt: ctx.now() }); return row;
    }
  };
  for (const [command, deleted] of [['visit_template_delete', true], ['visit_template_restore', false]] as const) handlers[command] = args => {
    const row = get((args as { id: number }).id); if (Boolean(row.deletedAt) === deleted) ctx.fail('conflict', 'Record conflicts with current state.');
    row.updatedAt = ctx.now(); row.deletedAt = deleted ? row.updatedAt : null; return row;
  };
  return { has: (command: string) => command in handlers, call: (command: string, args: unknown) => handlers[command](args) };
}
