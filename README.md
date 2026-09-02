# Financialy

MVP de organização financeira com React, Rust, PostgreSQL e autenticação exclusiva pelo Google. O usuário registra entradas e saídas e acompanha saldo, fluxo mensal e gastos por categoria.

## Implementado

- Login Google sem armazenamento de senhas.
- Sessão de sete dias em cookie `HttpOnly` e `SameSite=Lax`.
- Isolamento dos dados por usuário e categorias iniciais no primeiro login.
- CRUD completo de categorias e movimentações na API.
- Dashboard responsivo com totais e gráficos.
- Relatório detalhado em tabela com filtros, ordenação, pesquisa, criação, edição, exclusão e exportação CSV.
- Valores monetários armazenados em centavos e migrations PostgreSQL.
- Imagens Docker para frontend e API.

## Configuração do Google

1. No Google Cloud Console, crie ou selecione um projeto.
2. Configure a tela de consentimento OAuth.
3. Em **APIs e serviços → Credenciais**, crie um **ID do cliente OAuth** do tipo Aplicativo da Web.
4. Adicione `http://localhost:5173` e `http://localhost:8080` às origens JavaScript autorizadas.
5. Copie `.env.example` para `.env` e preencha `GOOGLE_CLIENT_ID`.
6. Gere `JWT_SECRET`, por exemplo com `openssl rand -hex 32`.

O frontend e a API devem usar o mesmo `GOOGLE_CLIENT_ID`. O backend valida o token e seu campo `aud` antes de criar uma sessão.

## Desenvolvimento local

Inicie o banco:

```bash
docker run --name financialy-db -e POSTGRES_USER=financialy -e POSTGRES_PASSWORD=financialy -e POSTGRES_DB=financialy -p 5432:5432 -d postgres:17-alpine
```

Copie as configurações e acrescente `VITE_GOOGLE_CLIENT_ID` ao arquivo do frontend:

```bash
cp .env.example .env
cp .env.example frontend/.env
printf '\nVITE_GOOGLE_CLIENT_ID=seu-client-id.apps.googleusercontent.com\n' >> frontend/.env
```

Execute em terminais separados:

```bash
cd backend && cargo run
cd frontend && npm install && npm run dev
```

Acesse `http://localhost:5173`.

## Docker Compose

```bash
cp .env.example .env
docker compose up --build
```

A aplicação estará em `http://localhost:8080`. Nesta máquina, o Docker está instalado, mas o plugin `docker compose` ainda precisa ser adicionado.

## Verificações

```bash
cd backend && cargo fmt --check && cargo test
cd frontend && npm run build
```

Em produção, use HTTPS, `COOKIE_SECURE=true`, novos segredos e a origem pública correta em `FRONTEND_URL` e no Google Cloud.
