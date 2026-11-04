import { tr } from '$lib/i18n/tr';
import type { PaymentMethod, ServiceCategory } from '$lib/api/types';
export const paymentLabels: Record<PaymentMethod, string> = { cash: tr.billingCashMethod, card: tr.billingCardMethod, transfer: tr.billingTransferMethod, other: tr.billingOtherMethod };
export const categoryLabels: Record<ServiceCategory, string> = { exam: tr.billingExam, vaccination: tr.billingVaccination, surgery: tr.billingSurgery, lab: tr.billingLab, imaging: tr.billingImaging, grooming: tr.billingGrooming, boarding: tr.billingBoarding, hospitalisation: tr.billingHospitalisation, other: tr.billingOther };
export const methodOptions = Object.entries(paymentLabels).map(([value,label]) => ({value,label}));
export const categoryOptions = Object.entries(categoryLabels).map(([value,label]) => ({value,label}));
