import { useMemo, useState } from 'react';
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import {
  ArrowDownUp,
  Download,
  FileSpreadsheet,
  Pencil,
  Plus,
  Search,
  Trash2,
} from 'lucide-react';
import { api } from '../api';
import { formatDate, formatMoney } from '../format';
import { buildReportCsv } from '../reportCsv';
import type { Category, Kind, Transaction } from '../types';

type SortKey = 'transaction_date' | 'description' | 'category_name' | 'kind' | 'amount_cents';

function iso(date: Date) {
  const year = date.getFullYear();
  const month = String(date.getMonth() + 1).padStart(2, '0');
  const day = String(date.getDate()).padStart(2, '0');
  return `${year}-${month}-${day}`;
}

function currentMonth() {
  const today = new Date();
  return {
    start: iso(new Date(today.getFullYear(), today.getMonth(), 1)),
    end: iso(new Date(today.getFullYear(), today.getMonth() + 1, 0)),
  };
}

export function DetailedTable({
  categories,
  onCreate,
  onEdit,
}: {
  categories: Category[];
  onCreate: () => void;
  onEdit: (transaction: Transaction) => void;
}) {
  const qc = useQueryClient();
  const initial = currentMonth();
  const [start, setStart] = useState(initial.start);
  const [end, setEnd] = useState(initial.end);
  const [kind, setKind] = useState<Kind | ''>('');
  const [categoryId, setCategoryId] = useState('');
  const [search, setSearch] = useState('');
  const [sort, setSort] = useState<SortKey>('transaction_date');
  const [ascending, setAscending] = useState(false);

  const report = useQuery({
    queryKey: ['detailed-report', start, end, kind, categoryId],
    queryFn: () => api.detailedReport({
      start,
      end,
      kind: kind || undefined,
      category_id: categoryId || undefined,
    }),
    enabled: Boolean(start && end && start <= end),
  });

  const remove = useMutation({
    mutationFn: api.deleteTransaction,
    onSuccess: async () => {
      await Promise.all([
        qc.invalidateQueries({ queryKey: ['detailed-report'] }),
        qc.invalidateQueries({ queryKey: ['dashboard'] }),
        qc.invalidateQueries({ queryKey: ['transactions'] }),
      ]);
    },
  });

  const rows = useMemo(() => {
    const term = search.trim().toLocaleLowerCase('pt-BR');
    const filtered = (report.data?.transactions ?? []).filter((transaction) =>
      !term || [transaction.description, transaction.category_name, transaction.notes]
        .some((value) => value?.toLocaleLowerCase('pt-BR').includes(term)),
    );
    return [...filtered].sort((left, right) => {
      const a = left[sort] ?? '';
      const b = right[sort] ?? '';
      const result = typeof a === 'number' && typeof b === 'number'
        ? a - b
        : String(a).localeCompare(String(b), 'pt-BR');
      return ascending ? result : -result;
    });
  }, [ascending, report.data?.transactions, search, sort]);

  function changeSort(next: SortKey) {
    if (sort === next) setAscending((value) => !value);
    else {
      setSort(next);
      setAscending(true);
    }
  }

  function exportCsv() {
    if (!report.data) return;
    const csv = buildReportCsv({ ...report.data, transactions: rows });
    const link = document.createElement('a');
    link.href = URL.createObjectURL(new Blob([csv], { type: 'text/csv;charset=utf-8' }));
    link.download = `financialy-${start}-${end}.csv`;
    link.click();
    URL.revokeObjectURL(link.href);
  }

  function deleteRow(transaction: Transaction) {
    if (confirm(`Excluir a movimentação “${transaction.description}”?`)) {
      remove.mutate(transaction.id);
    }
  }

  const availableCategories = categories.filter((category) => !kind || category.kind === kind);
  const invalidPeriod = start > end;
  const data = report.data;

  return <section className="details-view">
    <header className="details-heading">
      <div>
        <span className="details-kicker"><FileSpreadsheet /> Dados financeiros</span>
        <h2>Relatório detalhado</h2>
        <p>Consulte e gerencie todos os lançamentos em formato de tabela.</p>
      </div>
      <div className="details-actions">
        <button className="button secondary" onClick={exportCsv} disabled={!rows.length}>
          <Download /> Exportar CSV
        </button>
        <button className="button primary" onClick={onCreate}><Plus /> Nova movimentação</button>
      </div>
    </header>

    <div className="report-filters panel">
      <label>De<input type="date" value={start} onChange={(event) => setStart(event.target.value)} /></label>
      <label>Até<input type="date" value={end} onChange={(event) => setEnd(event.target.value)} /></label>
      <label>Tipo<select value={kind} onChange={(event) => { setKind(event.target.value as Kind | ''); setCategoryId(''); }}>
        <option value="">Todos</option><option value="income">Entradas</option><option value="expense">Saídas</option>
      </select></label>
      <label>Categoria<select value={categoryId} onChange={(event) => setCategoryId(event.target.value)}>
        <option value="">Todas</option>{availableCategories.map((category) => <option key={category.id} value={category.id}>{category.name}</option>)}
      </select></label>
      <label className="search-field">Pesquisar<span><Search /><input value={search} onChange={(event) => setSearch(event.target.value)} placeholder="Descrição, categoria..." /></span></label>
    </div>

    {invalidPeriod && <div className="error-banner">A data inicial não pode ser posterior à data final.</div>}
    {report.isError && <div className="error-banner">Não foi possível carregar o relatório.</div>}

    <div className="report-metrics">
      <Metric label="Entradas" value={formatMoney(data?.income_cents ?? 0)} tone="income" />
      <Metric label="Saídas" value={formatMoney(data?.expense_cents ?? 0)} tone="expense" />
      <Metric label="Saldo" value={formatMoney(data?.balance_cents ?? 0)} tone="balance" />
      <Metric label="Lançamentos" value={String(data?.transaction_count ?? 0)} subtitle={`Média ${formatMoney(data?.average_cents ?? 0)}`} />
    </div>

    <div className="details-table-panel panel">
      <div className="table-caption"><strong>{rows.length} registro{rows.length === 1 ? '' : 's'}</strong><span>Os dados do dashboard vêm destes lançamentos.</span></div>
      <div className="table-scroll">
        <table className="details-table">
          <thead><tr>
            <SortHead label="Data" field="transaction_date" onSort={changeSort} />
            <SortHead label="Descrição" field="description" onSort={changeSort} />
            <SortHead label="Categoria" field="category_name" onSort={changeSort} />
            <SortHead label="Tipo" field="kind" onSort={changeSort} />
            <SortHead label="Valor" field="amount_cents" onSort={changeSort} align="right" />
            <th>Observações</th><th aria-label="Ações" />
          </tr></thead>
          <tbody>
            {rows.map((transaction) => <tr key={transaction.id}>
              <td className="date-cell">{formatDate(transaction.transaction_date)}</td>
              <td><strong>{transaction.description}</strong></td>
              <td>{transaction.category_name ?? 'Sem categoria'}</td>
              <td><span className={`kind-badge ${transaction.kind}`}>{transaction.kind === 'income' ? 'Entrada' : 'Saída'}</span></td>
              <td className={`money-cell ${transaction.kind}`}>{transaction.kind === 'expense' ? '− ' : '+ '}{formatMoney(transaction.amount_cents)}</td>
              <td className="notes-cell" title={transaction.notes}>{transaction.notes || '—'}</td>
              <td><div className="row-actions">
                <button onClick={() => onEdit(transaction)} aria-label={`Editar ${transaction.description}`}><Pencil /></button>
                <button className="danger" onClick={() => deleteRow(transaction)} disabled={remove.isPending} aria-label={`Excluir ${transaction.description}`}><Trash2 /></button>
              </div></td>
            </tr>)}
          </tbody>
        </table>
        {!report.isLoading && !rows.length && <div className="table-empty"><FileSpreadsheet /><strong>Nenhum lançamento encontrado</strong><p>Ajuste os filtros ou crie uma nova movimentação.</p></div>}
        {report.isLoading && <div className="table-empty"><span className="table-loader" /><p>Carregando relatório...</p></div>}
      </div>
    </div>
  </section>;
}

function SortHead({ label, field, onSort, align }: {label:string;field:SortKey;onSort:(field:SortKey)=>void;align?:'right'}) {
  return <th className={align === 'right' ? 'align-right' : ''}><button onClick={() => onSort(field)}>{label}<ArrowDownUp /></button></th>;
}

function Metric({ label, value, subtitle, tone = '' }: {label:string;value:string;subtitle?:string;tone?:string}) {
  return <article className={`report-metric ${tone}`}><span>{label}</span><strong>{value}</strong>{subtitle && <small>{subtitle}</small>}</article>;
}
