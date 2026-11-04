import { call } from './client';
import type { NewVisitTemplate, VisitTemplatePatch } from './types';
export const list = (includeDeleted = false) => call('visit_templates_list', { includeDeleted });
export const get = (id: number) => call('visit_template_get', { id });
export const create = (input: NewVisitTemplate) => call('visit_template_create', { input });
export const update = (id: number, patch: VisitTemplatePatch) => call('visit_template_update', { id, patch });
export const softDelete = (id: number) => call('visit_template_delete', { id });
export const restore = (id: number) => call('visit_template_restore', { id });
