import type { DetailedReport } from './types';

function cell(value: string | number) {
  return `"${String(value).replaceAll('"', '""')}"`;
}

export function buildReportCsv(report: DetailedReport) {
  const rows = [
    ['Data', 'Descrição', 'Categoria', 'Tipo', 'Valor', 'Observações'],
    ...report.transactions.map((transaction) => [
      transaction.transaction_date,
      transaction.description,
      transaction.category_name ?? 'Sem categoria',
      transaction.kind === 'income' ? 'Entrada' : 'Saída',
      (transaction.amount_cents / 100).toFixed(2).replace('.', ','),
      transaction.notes ?? '',
    ]),
  ];
  return `\uFEFF${rows.map((row) => row.map(cell).join(';')).join('\r\n')}`;
}
