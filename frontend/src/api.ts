import type {Category,Dashboard,DetailedReport,ReportFilters,Transaction,TransactionInput,User} from './types';
async function request<T>(path:string,options?:RequestInit):Promise<T>{const response=await fetch(`/api${path}`,{credentials:'include',headers:{'Content-Type':'application/json',...options?.headers},...options});if(!response.ok){const body=await response.json().catch(()=>({}));throw new Error(body.error||'Não foi possível concluir a operação.')}if(response.status===204||response.headers.get('content-length')==='0')return undefined as T;return response.json()}
export const api={
 me:()=>request<User>('/auth/me'),login:(credential:string)=>request<User>('/auth/google',{method:'POST',body:JSON.stringify({credential})}),logout:()=>request<void>('/auth/logout',{method:'POST'}),
 categories:()=>request<Category[]>('/categories'),createCategory:(data:Omit<Category,'id'|'created_at'>)=>request<Category>('/categories',{method:'POST',body:JSON.stringify(data)}),
 transactions:()=>request<Transaction[]>('/transactions'),createTransaction:(data:TransactionInput)=>request<Transaction>('/transactions',{method:'POST',body:JSON.stringify(data)}),updateTransaction:(id:string,data:TransactionInput)=>request<Transaction>(`/transactions/${id}`,{method:'PUT',body:JSON.stringify(data)}),deleteTransaction:(id:string)=>request<void>(`/transactions/${id}`,{method:'DELETE'}),
 dashboard:(start:string,end:string)=>request<Dashboard>(`/dashboard?start=${start}&end=${end}`),
 detailedReport:(filters:ReportFilters)=>{const params=new URLSearchParams({start:filters.start,end:filters.end});if(filters.kind)params.set('kind',filters.kind);if(filters.category_id)params.set('category_id',filters.category_id);return request<DetailedReport>(`/reports/detailed?${params}`)}
};
