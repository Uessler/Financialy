import { describe, expect, it } from 'vitest';
import { formatDate, formatMoney, monthPeriod } from './format';

describe('financial formatters', () => {
  it('formats cents as Brazilian reais', () => {
    expect(formatMoney(123456)).toMatch(/R\$\s*1\.234,56/);
  });

  it('formats an ISO date without shifting its calendar day', () => {
    expect(formatDate('2026-09-02')).toMatch(/^02 de set\.?$/);
  });

  it('returns the complete month across a year boundary', () => {
    expect(monthPeriod(1, new Date(2026, 11, 15))).toMatchObject({
      start: '2027-01-01',
      end: '2027-01-31',
    });
  });

  it('handles leap years', () => {
    expect(monthPeriod(0, new Date(2028, 1, 10)).end).toBe('2028-02-29');
  });
});
