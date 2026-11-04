import type { ApiError, Audit, Charge, ChargeSource, CommandMap, DailySummary, NewCharge, NewPayment, NewService, Owner, Patient, Payment, Receivable, Service, StatementEntry, Visit } from './types';
import { tr } from '$lib/i18n/tr';
import { dateAfterDays, isValidDate, todayDate } from '$lib/format';

type Deps = {
  owners:()=>Owner[]; ownerGet:(id:number)=>Owner; patientGet:(id:number)=>Patient; visitGet:(id:number)=>Visit;
  now:()=>string; fail:(kind:ApiError['kind'],message:string,field?:string)=>never;
};
export function createBillingMock(deps:Deps) {
  let nextId=1;
  const services:Service[]=[];
  const charges:Charge[]=[];
  const payments:Payment[]=[];
  const {fail,now}=deps;
  function required(field:string,value:string){if(typeof value!=='string'||!value.trim())fail('invalid','Must not be empty.',field);}
  function money(field:string,value:number|null){if(value!==null&&(!Number.isSafeInteger(value)||value<0))fail('invalid','Invalid money.',field);}
  function vat(value:number|null){if(value!==null&&(!Number.isInteger(value)||value<0||value>100))fail('invalid','Invalid VAT.','vatRate');}
  function date(field:string,value:string){if(!isValidDate(value))fail('invalid','Invalid date.',field);}
  function activeOwner(id:number){const o=deps.ownerGet(id);if(o.deletedAt)fail('not_found','Record not found.');return o;}
  function audit():Audit{const stamp=now();return{id:nextId++,createdAt:stamp,updatedAt:stamp,deletedAt:null};}
  function get<T extends Audit>(rows:T[],id:number):T{return rows.find(r=>r.id===id)??fail('not_found','Record not found.');}
  function active<T extends Audit>(rows:T[],id:number):T{const row=get(rows,id);if(row.deletedAt)fail('not_found','Record not found.');return row;}
  function deleted<T extends Audit>(rows:T[],id:number,value:boolean):T{const row=get(rows,id);if(!!row.deletedAt===value)fail('conflict','Record conflicts with its current state.');row.updatedAt=now();row.deletedAt=value?row.updatedAt:null;return row;}
  function serviceInput(input:NewService){required('name',input.name);money('priceMinor',input.priceMinor);vat(input.vatRate);if(!['exam','vaccination','surgery','lab','imaging','grooming','boarding','hospitalisation','other'].includes(input.category))fail('invalid','Unknown category.');}
  function chargeTotal(input:NewCharge){
    required('description',input.description);activeOwner(input.ownerId);date('chargedOn',input.chargedOn);
    money('unitPriceMinor',input.unitPriceMinor);money('discountMinor',input.discountMinor);vat(input.vatRate);
    const gross=Math.round(input.quantity*input.unitPriceMinor);
    if(!Number.isFinite(input.quantity)||input.quantity<=0||!Number.isSafeInteger(gross)||input.discountMinor>gross)fail('invalid','Invalid charge.','discountMinor');
    if(input.patientId!==null){const patient=deps.patientGet(input.patientId);if(patient.deletedAt||patient.ownerId!==input.ownerId)fail('invalid','Patient must belong to owner.','patientId');}
    if(input.visitId!==null){const visit=deps.visitGet(input.visitId);const patient=deps.patientGet(visit.patientId);if(visit.deletedAt||patient.deletedAt||patient.ownerId!==input.ownerId||(input.patientId!==null&&input.patientId!==visit.patientId))fail('invalid','Visit must match owner and patient.','visitId');}
    if(input.serviceId!==null)active(services,input.serviceId);
    if(input.inventoryItemId!==null&&!sources.some(s=>s.inventoryItemId===input.inventoryItemId))fail('not_found','Record not found.');
    return gross-input.discountMinor;
  }
  function paymentInput(input:NewPayment){activeOwner(input.ownerId);date('paidOn',input.paidOn);money('amountMinor',input.amountMinor);if(input.amountMinor<=0)fail('invalid','Payment must be positive.','amountMinor');if(!['cash','card','transfer','other'].includes(input.method))fail('invalid','Unknown method.','method');}
  function serviceCreate(input:NewService){serviceInput(input);const row:Service={...input,...audit(),isBuiltin:false};services.push(row);return row;}
  function chargeCreate(input:NewCharge){const totalMinor=chargeTotal(input);const row:Charge={...input,...audit(),totalMinor};charges.push(row);return row;}
  function paymentCreate(input:NewPayment){paymentInput(input);const row:Payment={...input,...audit()};payments.push(row);return row;}
  function ledger(ownerId?:number,includeDeleted=false):StatementEntry[]{
    const rows=[...charges.map(c=>({entry:{id:c.id,kind:'charge' as const,ownerId:c.ownerId,ownerName:deps.ownerGet(c.ownerId).fullName,date:c.chargedOn,description:c.description,amountMinor:c.totalMinor,method:null,notes:c.notes,deletedAt:c.deletedAt,runningBalanceMinor:0},createdAt:c.createdAt})),
      ...payments.map(p=>({entry:{id:p.id,kind:'payment' as const,ownerId:p.ownerId,ownerName:deps.ownerGet(p.ownerId).fullName,date:p.paidOn,description:'',amountMinor:p.amountMinor,method:p.method,notes:p.notes,deletedAt:p.deletedAt,runningBalanceMinor:0},createdAt:p.createdAt}))]
      .filter(r=>(ownerId===undefined||r.entry.ownerId===ownerId)&&(includeDeleted||!r.entry.deletedAt))
      .sort((a,b)=>a.entry.date.localeCompare(b.entry.date)||a.createdAt.localeCompare(b.createdAt)||a.entry.kind.localeCompare(b.entry.kind)||a.entry.id-b.entry.id);
    const balances=new Map<number,number>();
    return rows.map(({entry})=>{const balance=(balances.get(entry.ownerId)??0)+(entry.deletedAt?0:entry.kind==='charge'?entry.amountMinor:-entry.amountMinor);if(!Number.isSafeInteger(balance))fail('invalid','Account total is too large.');balances.set(entry.ownerId,balance);return{...entry,runningBalanceMinor:balance};});
  }
  function statement(ownerId:number,includeDeleted=false){deps.ownerGet(ownerId);return ledger(ownerId,includeDeleted);}
  function balance(ownerId:number){return statement(ownerId).at(-1)?.runningBalanceMinor??0;}
  function receivables():Receivable[]{
    return deps.owners().flatMap(owner=>{
      const rows=statement(owner.id);const current=rows.at(-1)?.runningBalanceMinor??0;if(current<=0)return[];
      const paid=rows.filter(r=>r.kind==='payment');let remaining=paid.reduce((sum,p)=>sum+p.amountMinor,0);
      for(const charge of rows.filter(r=>r.kind==='charge')){if(remaining>=charge.amountMinor)remaining-=charge.amountMinor;else return[{ownerId:owner.id,ownerName:owner.fullName,phone:owner.phone,balanceMinor:current,oldestUnpaidOn:charge.date,lastPaymentOn:paid.at(-1)?.date??null}];}
      return[];
    }).sort((a,b)=>b.balanceMinor-a.balanceMinor||a.oldestUnpaidOn.localeCompare(b.oldestUnpaidOn)||a.ownerId-b.ownerId);
  }
  function summary(from:string,to:string):DailySummary{
    date('from',from);date('to',to);if(from>to)fail('invalid','Invalid date range.','to');
    const entries=ledger().filter(r=>r.date>=from&&r.date<=to).map(r=>({...r,runningBalanceMinor:0}));
    const result:DailySummary={chargesMinor:0,collectionsMinor:0,collectionsByMethod:{cash:0,card:0,transfer:0,other:0},entries};
    for(const row of entries)if(row.kind==='charge')result.chargesMinor+=row.amountMinor;else{result.collectionsMinor+=row.amountMinor;if(row.method)result.collectionsByMethod[row.method]+=row.amountMinor;}
    return result;
  }
  // Synthetic catalog choices: one linked price and one requiring a manual price.
  const sources:ChargeSource[]=[{key:'medication-1-1',name:tr.medications,inventoryItemId:1,priceMinor:45000},{key:'vaccine-1-0',name:tr.billingVaccination,inventoryItemId:null,priceMinor:null}];
  serviceCreate({name:tr.billingExam,category:'exam',priceMinor:125000,vatRate:20,isActive:true,notes:null}).isBuiltin=true;
  serviceCreate({name:tr.billingVaccination,category:'vaccination',priceMinor:80000,vatRate:20,isActive:true,notes:null}).isBuiltin=true;
  serviceCreate({name:tr.billingLab,category:'lab',priceMinor:null,vatRate:null,isActive:true,notes:null}).isBuiltin=true;
  const demo=deps.owners();
  for(const [index,owner] of demo.entries()){
    chargeCreate({ownerId:owner.id,patientId:null,visitId:null,serviceId:services[0].id,inventoryItemId:null,description:tr.billingExam,quantity:1,unitPriceMinor:125000,discountMinor:0,vatRate:20,chargedOn:dateAfterDays(todayDate(),-35+index*10),notes:null});
    if(index===0)paymentCreate({ownerId:owner.id,paidOn:todayDate(),amountMinor:25000,method:'cash',notes:null});
    if(index===2)paymentCreate({ownerId:owner.id,paidOn:todayDate(),amountMinor:150000,method:'card',notes:null});
  }
  if(demo[0])chargeCreate({ownerId:demo[0].id,patientId:null,visitId:null,serviceId:services[1].id,inventoryItemId:null,description:tr.billingVaccination,quantity:1,unitPriceMinor:80000,discountMinor:0,vatRate:20,chargedOn:todayDate(),notes:null});
  const names=new Set(['service_get','service_create','service_update','service_delete','service_restore','services_list','charge_get','charge_create','charge_update','charge_delete','charge_restore','charges_list','charges_by_visit','payment_get','payment_create','payment_update','payment_delete','payment_restore','payments_list','account_balance','account_statement','accounts_receivables','billing_daily_summary','billing_charge_sources']);
  return {
    has:(command:string)=>names.has(command),
    call(command:keyof CommandMap,args:CommandMap[keyof CommandMap]['args']):unknown{
      // Public CommandMap checks the dispatch boundary; each command validates its own fields.
      const a=args as {id:number;input:NewCharge&NewPayment&NewService;ownerId:number;visitId:number;includeDeleted:boolean;from:string;to:string};
      switch(command){
        case 'services_list':return services.filter(r=>a.includeDeleted||!r.deletedAt);
        case 'service_get':return get(services,a.id);
        case 'service_create':return serviceCreate(a.input);
        case 'service_update':{const r=active(services,a.id);serviceInput(a.input);Object.assign(r,a.input,{updatedAt:now()});return r;}
        case 'service_delete':return deleted(services,a.id,true);
        case 'service_restore':return deleted(services,a.id,false);
        case 'charges_list':return charges.filter(r=>a.includeDeleted||!r.deletedAt);
        case 'charges_by_visit':return charges.filter(r=>r.visitId===a.visitId&&(a.includeDeleted||!r.deletedAt)).sort((a,b)=>a.chargedOn.localeCompare(b.chargedOn)||a.id-b.id);
        case 'charge_get':return get(charges,a.id);
        case 'charge_create':return chargeCreate(a.input);
        case 'charge_update':{const r=active(charges,a.id);const totalMinor=chargeTotal(a.input);Object.assign(r,a.input,{totalMinor,updatedAt:now()});return r;}
        case 'charge_delete':return deleted(charges,a.id,true);
        case 'charge_restore':return deleted(charges,a.id,false);
        case 'payments_list':return payments.filter(r=>a.includeDeleted||!r.deletedAt);
        case 'payment_get':return get(payments,a.id);
        case 'payment_create':return paymentCreate(a.input);
        case 'payment_update':{const r=active(payments,a.id);paymentInput(a.input);Object.assign(r,a.input,{updatedAt:now()});return r;}
        case 'payment_delete':return deleted(payments,a.id,true);
        case 'payment_restore':return deleted(payments,a.id,false);
        case 'account_balance':return balance(a.ownerId);
        case 'account_statement':return statement(a.ownerId,a.includeDeleted);
        case 'accounts_receivables':return receivables();
        case 'billing_daily_summary':return summary(a.from,a.to);
        case 'billing_charge_sources':return sources;
        default:return fail('internal','Unknown billing command.');
      }
    }
  };
}
