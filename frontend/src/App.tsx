import { useEffect, useState } from 'react';
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import {
  BarChart3, Bell, ChevronLeft, ChevronRight, CircleDollarSign, FileSpreadsheet,
  LayoutDashboard, LogOut, Menu, Moon, Plus, ReceiptText, Settings, Sun, Tags,
  TrendingDown, TrendingUp, WalletCards,
} from 'lucide-react';
import {
  Bar, BarChart, CartesianGrid, Cell, Pie, PieChart, ResponsiveContainer,
  Tooltip, XAxis, YAxis,
} from 'recharts';
import { api } from './api';
import { DetailedTable } from './components/DetailedTable';
import { Login } from './components/Login';
import { TransactionModal } from './components/TransactionModal';
import { formatDate, formatMoney, monthPeriod } from './format';
import type { Transaction, TransactionInput, User } from './types';

type View = 'dashboard' | 'details';

export default function App() {
  const qc = useQueryClient();
  const [offset, setOffset] = useState(0);
  const [modal, setModal] = useState(false);
  const [editing, setEditing] = useState<Transaction>();
  const [menu, setMenu] = useState(false);
  const [view, setView] = useState<View>('dashboard');
  const [dark, setDark] = useState(() => localStorage.getItem('financialy-theme') === 'dark'
    || (!localStorage.getItem('financialy-theme') && matchMedia('(prefers-color-scheme: dark)').matches));
  const period = monthPeriod(offset);
  const me = useQuery({ queryKey: ['me'], queryFn: api.me, retry: false });
  const login = useMutation({ mutationFn: api.login, onSuccess: (user) => qc.setQueryData(['me'], user) });

  useEffect(() => {
    document.documentElement.dataset.theme = dark ? 'dark' : 'light';
    localStorage.setItem('financialy-theme', dark ? 'dark' : 'light');
  }, [dark]);

  if (me.isLoading) return <div className="center-loader"><span className="brand-mark"><BarChart3 /></span></div>;
  if (!me.data) return <><ThemeToggle dark={dark} setDark={setDark} /><Login onCredential={(value) => login.mutate(value)} error={login.error?.message} /></>;

  return <><ThemeToggle dark={dark} setDark={setDark} /><Dashboard
    user={me.data} period={period} setOffset={setOffset} modal={modal} setModal={setModal}
    editing={editing} setEditing={setEditing} menu={menu} setMenu={setMenu} view={view} setView={setView}
  /></>;
}

function ThemeToggle({ dark, setDark }: {dark:boolean;setDark:React.Dispatch<React.SetStateAction<boolean>>}) {
  return <button className="icon-btn global-theme-toggle" onClick={() => setDark((value) => !value)} aria-label={dark ? 'Ativar tema claro' : 'Ativar tema escuro'} title={dark ? 'Tema claro' : 'Tema escuro'}>{dark ? <Sun /> : <Moon />}</button>;
}

type DashboardProps = {
  user: User;
  period: ReturnType<typeof monthPeriod>;
  setOffset: React.Dispatch<React.SetStateAction<number>>;
  modal: boolean;
  setModal: React.Dispatch<React.SetStateAction<boolean>>;
  editing?: Transaction;
  setEditing: React.Dispatch<React.SetStateAction<Transaction | undefined>>;
  menu: boolean;
  setMenu: React.Dispatch<React.SetStateAction<boolean>>;
  view: View;
  setView: React.Dispatch<React.SetStateAction<View>>;
};

function Dashboard({ user, period, setOffset, modal, setModal, editing, setEditing, menu, setMenu, view, setView }: DashboardProps) {
  const qc = useQueryClient();
  const summary = useQuery({ queryKey: ['dashboard', period.start], queryFn: () => api.dashboard(period.start, period.end) });
  const transactions = useQuery({ queryKey: ['transactions'], queryFn: api.transactions });
  const categories = useQuery({ queryKey: ['categories'], queryFn: api.categories });
  const save = useMutation({
    mutationFn: (value: TransactionInput) => editing ? api.updateTransaction(editing.id, value) : api.createTransaction(value),
    onSuccess: async () => {
      setModal(false);
      setEditing(undefined);
      await Promise.all([
        qc.invalidateQueries({ queryKey: ['dashboard'] }),
        qc.invalidateQueries({ queryKey: ['transactions'] }),
        qc.invalidateQueries({ queryKey: ['detailed-report'] }),
      ]);
    },
  });
  const logout = useMutation({ mutationFn: api.logout, onSuccess: () => { qc.clear(); location.reload(); } });

  function navigate(next: View) {
    setView(next);
    setMenu(false);
  }
  function createTransaction() {
    setEditing(undefined);
    setModal(true);
  }
  function editTransaction(transaction: Transaction) {
    setEditing(transaction);
    setModal(true);
  }

  const dashboard = summary.data;
  const chart = dashboard?.monthly.map((point) => ({
    ...point,
    label: new Intl.DateTimeFormat('pt-BR', { month: 'short' }).format(new Date(`${point.month}T12:00:00`)),
    Entradas: point.income_cents / 100,
    Saídas: point.expense_cents / 100,
  })) ?? [];

  return <div className="app-shell">
    <aside className={menu ? 'sidebar open' : 'sidebar'}>
      <div className="brand"><span className="brand-mark"><BarChart3 /></span>Financialy</div>
      <nav>
        <button className={view === 'dashboard' ? 'active' : ''} onClick={() => navigate('dashboard')}><LayoutDashboard />Visão geral</button>
        <button className={view === 'details' ? 'active' : ''} onClick={() => navigate('details')}><FileSpreadsheet />Detalhado</button>
        <button disabled><Tags />Categorias</button>
        <button disabled><WalletCards />Orçamentos <small>Em breve</small></button>
      </nav>
      <div className="sidebar-bottom">
        <button disabled><Settings />Configurações</button>
        <button onClick={() => logout.mutate()}><LogOut />Sair</button>
        <div className="user-card">{user.avatar_url ? <img src={user.avatar_url} alt="" /> : <span>{user.name[0]}</span>}<div><strong>{user.name}</strong><small>{user.email}</small></div></div>
      </div>
    </aside>

    <main className="content">
      <header className="topbar">
        <button className="mobile-menu" onClick={() => setMenu(!menu)}><Menu /></button>
        <div><h1>{view === 'dashboard' ? <>Olá, {user.name.split(' ')[0]}! <span>👋</span></> : 'Detalhado'}</h1><p>{view === 'dashboard' ? 'Acompanhe suas finanças e fique no controle.' : 'Sua base completa de movimentações.'}</p></div>
        <div className="top-actions"><button className="icon-btn"><Bell /></button>{view === 'dashboard' && <button className="button primary" onClick={createTransaction}><Plus />Nova movimentação</button>}</div>
      </header>

      {view === 'dashboard' ? <>
        <section className="period"><button onClick={() => setOffset((value) => value - 1)}><ChevronLeft /></button><strong>{period.label}</strong><button onClick={() => setOffset((value) => value + 1)} disabled={period.start === monthPeriod(0).start}><ChevronRight /></button></section>
        {summary.isError ? <div className="error-banner">Não foi possível carregar o resumo. Verifique se a API está conectada.</div> : <>
          <section className="cards"><Stat title="Saldo do mês" value={formatMoney(dashboard?.balance_cents ?? 0)} icon={<CircleDollarSign />} tone="purple" /><Stat title="Entradas" value={formatMoney(dashboard?.income_cents ?? 0)} icon={<TrendingUp />} tone="green" /><Stat title="Saídas" value={formatMoney(dashboard?.expense_cents ?? 0)} icon={<TrendingDown />} tone="red" /></section>
          <section className="charts">
            <article className="panel cash"><PanelTitle title="Fluxo financeiro" subtitle="Entradas e saídas dos últimos meses" /><div className="chart-area">{chart.length ? <ResponsiveContainer width="100%" height="100%"><BarChart data={chart} accessibilityLayer={false}><CartesianGrid vertical={false} stroke="#edf0f4" /><XAxis dataKey="label" axisLine={false} tickLine={false} /><YAxis axisLine={false} tickLine={false} tickFormatter={(value) => `${value / 1000}k`} /><Tooltip cursor={{ fill: '#f5f4ff' }} animationDuration={180} contentStyle={{ background: '#fff', border: '1px solid #e2e0f4', borderRadius: 10, boxShadow: '0 8px 24px #25274a20', padding: '10px 12px' }} labelStyle={{ color: '#6e7380', fontWeight: 700, marginBottom: 6, textTransform: 'capitalize' }} formatter={(value, name) => [formatMoney(Number(value) * 100), String(name)]} /><Bar dataKey="Entradas" fill="#5b4de3" radius={[5, 5, 0, 0]} activeBar={{ fill: '#7164ea', stroke: '#493ac5', strokeWidth: 1 }} animationDuration={750} animationEasing="ease-out" /><Bar dataKey="Saídas" fill="#dcd9fa" radius={[5, 5, 0, 0]} activeBar={{ fill: '#c9c4f5', stroke: '#9e96e5', strokeWidth: 1 }} animationDuration={900} animationEasing="ease-out" /></BarChart></ResponsiveContainer> : <Empty text="Registre movimentações para ver sua evolução." />}</div></article>
            <article className="panel categories-chart"><PanelTitle title="Gastos por categoria" subtitle="Distribuição das suas saídas" /><div className="donut-row">{dashboard?.by_category.length ? <><div className="donut"><ResponsiveContainer><PieChart><Pie data={dashboard.by_category} dataKey="total_cents" innerRadius={58} outerRadius={82} paddingAngle={3}>{dashboard.by_category.map((item) => <Cell key={item.category} fill={item.color} />)}</Pie></PieChart></ResponsiveContainer></div><div className="legend">{dashboard.by_category.slice(0, 5).map((item) => <div key={item.category}><span style={{ background: item.color }} /><p>{item.category}<strong>{formatMoney(item.total_cents)}</strong></p></div>)}</div></> : <Empty text="Seus gastos aparecerão aqui." />}</div></article>
          </section>
        </>}
        <section className="panel recent"><PanelTitle title="Movimentações recentes" subtitle="Seus últimos lançamentos" action="Ver todas" onAction={() => navigate('details')} /><div className="transaction-list">{transactions.data?.slice(0, 6).map((transaction) => <div className="transaction" key={transaction.id}><span className={`transaction-icon ${transaction.kind}`}><ReceiptText /></span><div className="transaction-name"><strong>{transaction.description}</strong><small>{transaction.category_name ?? 'Sem categoria'}</small></div><span className={`amount ${transaction.kind}`}>{transaction.kind === 'expense' ? '− ' : '+ '}{formatMoney(transaction.amount_cents)}</span><time>{formatDate(transaction.transaction_date)}</time></div>)}{!transactions.isLoading && !transactions.data?.length && <Empty text="Nenhuma movimentação ainda. Comece adicionando a primeira." />}</div></section>
      </> : <DetailedTable categories={categories.data ?? []} onCreate={createTransaction} onEdit={editTransaction} />}
    </main>

    {modal && <TransactionModal key={editing?.id ?? 'new'} initial={editing} categories={categories.data ?? []} onClose={() => { setModal(false); setEditing(undefined); }} onSave={(value) => save.mutate(value)} busy={save.isPending} />}
  </div>;
}

function Stat({ title, value, icon, tone }: {title:string;value:string;icon:React.ReactNode;tone:string}) {
  return <article className="stat"><span className={`stat-icon ${tone}`}>{icon}</span><div><p>{title}</p><strong>{value}</strong><small><span className={tone === 'red' ? 'down' : ''}>{tone === 'red' ? 'Acompanhe seus gastos' : 'Visão do período'}</span></small></div></article>;
}

function PanelTitle({ title, subtitle, action, onAction }: {title:string;subtitle:string;action?:string;onAction?:()=>void}) {
  return <header className="panel-title"><div><h2>{title}</h2><p>{subtitle}</p></div>{action && <button onClick={onAction}>{action}<ChevronRight /></button>}</header>;
}

function Empty({ text }: {text:string}) {
  return <div className="empty"><BarChart3 /><p>{text}</p></div>;
}
