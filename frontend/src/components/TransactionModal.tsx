import { useMemo, useState, type FormEvent } from 'react';
import { X } from 'lucide-react';
import type { Category, Kind, Transaction, TransactionInput } from '../types';

export function TransactionModal({ categories, initial, onClose, onSave, busy }: {
  categories: Category[];
  initial?: Transaction;
  onClose: () => void;
  onSave: (value: TransactionInput) => void;
  busy: boolean;
}) {
  const [kind, setKind] = useState<Kind>(initial?.kind ?? 'expense');
  const [description, setDescription] = useState(initial?.description ?? '');
  const [amount, setAmount] = useState(initial ? (initial.amount_cents / 100).toFixed(2).replace('.', ',') : '');
  const [category, setCategory] = useState(initial?.category_id ?? '');
  const [date, setDate] = useState(initial?.transaction_date ?? new Date().toISOString().slice(0, 10));
  const [notes, setNotes] = useState(initial?.notes ?? '');
  const filtered = useMemo(() => categories.filter((item) => item.kind === kind), [categories, kind]);

  function submit(event: FormEvent) {
    event.preventDefault();
    const normalized = amount.replace(/\s/g, '').replace(/\.(?=.*[,])/, '').replace(',', '.');
    const cents = Math.round(Number(normalized) * 100);
    if (!description.trim() || !Number.isFinite(cents) || cents <= 0) return;
    onSave({ kind, description: description.trim(), amount_cents: cents, category_id: category || null, transaction_date: date, notes: notes.trim() || null });
  }

  return <div className="modal-backdrop" onMouseDown={(event) => event.currentTarget === event.target && onClose()}><form className="modal" onSubmit={submit}>
    <header><div><h2>{initial ? 'Editar movimentação' : 'Nova movimentação'}</h2><p>{initial ? 'Atualize os dados deste lançamento.' : 'Registre uma entrada ou uma saída.'}</p></div><button type="button" className="icon-btn" onClick={onClose} aria-label="Fechar"><X /></button></header>
    <div className="kind-switch"><button type="button" className={kind === 'expense' ? 'active expense' : ''} onClick={() => { setKind('expense'); setCategory(''); }}>Saída</button><button type="button" className={kind === 'income' ? 'active income' : ''} onClick={() => { setKind('income'); setCategory(''); }}>Entrada</button></div>
    <label>Descrição<input autoFocus required maxLength={120} value={description} onChange={(event) => setDescription(event.target.value)} placeholder="Ex.: Supermercado" /></label>
    <div className="form-grid"><label>Valor (R$)<input required inputMode="decimal" value={amount} onChange={(event) => setAmount(event.target.value)} placeholder="0,00" /></label><label>Data<input required type="date" value={date} onChange={(event) => setDate(event.target.value)} /></label></div>
    <label>Categoria<select value={category} onChange={(event) => setCategory(event.target.value)}><option value="">Sem categoria</option>{filtered.map((item) => <option value={item.id} key={item.id}>{item.name}</option>)}</select></label>
    <label>Observação <span>(opcional)</span><textarea maxLength={500} value={notes} onChange={(event) => setNotes(event.target.value)} placeholder="Adicione mais detalhes..." /></label>
    <footer><button type="button" className="button secondary" onClick={onClose}>Cancelar</button><button className="button primary" disabled={busy}>{busy ? 'Salvando...' : initial ? 'Salvar alterações' : 'Salvar movimentação'}</button></footer>
  </form></div>;
}
