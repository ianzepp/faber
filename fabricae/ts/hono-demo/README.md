# Hono Demo

A REST API demo written in Faber, compiled to TypeScript, running on Bun.

## Build Pipeline

```
fons/*.fab  →  Faber Compiler  →  dist/*.ts  →  Bun Runtime
```

1. **Source**: Faber code in `fons/`
2. **Compile**: `bun run build` compiles `.fab` to TypeScript
3. **Run**: `bun run start` executes the compiled TypeScript

## Quick Start

```bash
# Install dependencies
bun install

# Build (compile Faber to TypeScript)
bun run build

# Run server
bun run start

# Or build + run with watch mode
bun run dev
```

## API Endpoints

| Method | Path         | Description    |
| ------ | ------------ | -------------- |
| GET    | `/users`     | List all users |
| GET    | `/users/:id` | Get user by ID |
| POST   | `/users`     | Create user    |
| PUT    | `/users/:id` | Update user    |
| DELETE | `/users/:id` | Delete user    |
| GET    | `/health`    | Health check   |

## Test with curl

```bash
# Health check
curl http://localhost:3000/health

# Create user
curl -X POST http://localhost:3000/users \
  -H "Content-Type: application/json" \
  -d '{"nomen": "Marcus", "email": "marcus@roma.it"}'

# List users
curl http://localhost:3000/users

# Get user
curl http://localhost:3000/users/1

# Update user
curl -X PUT http://localhost:3000/users/1 \
  -H "Content-Type: application/json" \
  -d '{"nomen": "Marcus Aurelius", "email": "emperor@roma.it"}'

# Delete user
curl -X DELETE http://localhost:3000/users/1
```

## Faber Features Demonstrated

- **Imports**: `ex "hono" importa Hono`
- **Sync lambdas**: `pro c: c.json(users)`
- **Async lambdas**: `fiet c { fixum body = cede c.req.json() }`
- **Collection methods**: `users.inveni()`, `users.adde()`, `users.inveniIndicem()`
- **Null checks**: `si user === nihil { ... }`
- **Type casting**: `[] ut lista<objectum>`
