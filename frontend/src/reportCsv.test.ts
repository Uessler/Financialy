import { describe, expect, it } from 'vitest';
import { buildReportCsv } from './reportCsv';
import type { DetailedReport } from './types';

describe('buildReportCsv', () => {
  it('exports Portuguese-compatible CSV and escapes quotes', () => {
    const report = {
      start: '2026-09-01', end: '2026-09-30', income_cents: 12345,
      expense_cents: 0, balance_cents: 12345, transaction_count: 1,
      average_cents: 12345, by_category: [],
      transactions: [{id:'1',kind:'income',description:'Venda "especial"',amount_cents:12345,transaction_date:'2026-09-02',created_at:'2026-09-02'}],
    } satisfies DetailedReport;
    const csv = buildReportCsv(report);
    expect(csv.startsWith('\uFEFF')).toBe(true);
    expect(csv).toContain('"Venda ""especial"""');
    expect(csv).toContain('"123,45"');
  });
});
