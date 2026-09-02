export type Kind='income'|'expense';
export interface User{id:string;email:string;name:string;avatar_url?:string}
export interface Category{id:string;name:string;color:string;kind:Kind;created_at:string}
export interface Transaction{id:string;category_id?:string;category_name?:string;kind:Kind;description:string;amount_cents:number;transaction_date:string;notes?:string;created_at:string}
export interface Dashboard{income_cents:number;expense_cents:number;balance_cents:number;monthly:{month:string;income_cents:number;expense_cents:number}[];by_category:{category:string;color:string;total_cents:number}[]}
export interface TransactionInput{category_id?:string|null;kind:Kind;description:string;amount_cents:number;transaction_date:string;notes?:string|null}
