# Sample App - Todo API

A minimal RustPress app example demonstrating routing and CRUD operations.

## Features

- **Router Setup**: Axum-based routing
- **CRUD Handlers**: List, Get, Create, Update, Delete
- **Validation**: Input validation with `validator`
- **Error Handling**: Consistent API error responses
- **Database**: SQLx with PostgreSQL

## API Endpoints

| Method | Path | Description |
|--------|------|-------------|
| GET | `/todos` | List all todos |
| GET | `/todos?completed=true` | Filter by status |
| GET | `/todos/:id` | Get single todo |
| POST | `/todos` | Create todo |
| PUT | `/todos/:id` | Update todo |
| DELETE | `/todos/:id` | Delete todo |

## Usage

```bash
# List todos
curl http://localhost:3000/todos

# Create todo
curl -X POST http://localhost:3000/todos \
  -H "Content-Type: application/json" \
  -d '{"title": "Learn RustPress"}'

# Update todo
curl -X PUT http://localhost:3000/todos/1 \
  -H "Content-Type: application/json" \
  -d '{"completed": true}'

# Delete todo
curl -X DELETE http://localhost:3000/todos/1
```

## Database Schema

```sql
CREATE TABLE todos (
    id BIGSERIAL PRIMARY KEY,
    title VARCHAR(200) NOT NULL,
    completed BOOLEAN DEFAULT false
);
```

## File Structure

```
sample-app/
├── Cargo.toml
└── src/
    └── main.rs    # Router, handlers, models, error handling
```
