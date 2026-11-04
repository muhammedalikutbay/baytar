import { call } from './client';
import type { DoseInput } from './types';

export const calculate = (input: DoseInput) => call('dose_calculate', { input });
