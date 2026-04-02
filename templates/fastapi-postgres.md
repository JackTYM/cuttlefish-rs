---
name: fastapi-postgres
description: FastAPI backend with PostgreSQL and SQLAlchemy
language: python
docker_image: python:3.12-slim
variables:
  - name: project_name
    description: Name of the project
    required: true
tags: [backend, python, api, database]
---

# {{ project_name }}

FastAPI backend with PostgreSQL and SQLAlchemy ORM.

## Project Structure

```
{{ project_name }}/
├── pyproject.toml
├── main.py
├── models.py
├── schemas.py
├── database.py
├── requirements.txt
├── docker-compose.yml
├── Dockerfile
├── alembic/
│   ├── env.py
│   ├── script.py.mako
│   └── versions/
├── app/
│   ├── __init__.py
│   ├── api/
│   │   ├── __init__.py
│   │   ├── routes.py
│   │   └── dependencies.py
│   └── crud/
│       ├── __init__.py
│       └── user.py
└── tests/
    ├── __init__.py
    └── test_api.py
```

## Files

### pyproject.toml
```toml
[project]
name = "{{ project_name }}"
version = "0.1.0"
description = "FastAPI application with PostgreSQL"
requires-python = ">=3.12"
dependencies = [
    "fastapi==0.104.1",
    "uvicorn[standard]==0.24.0",
    "sqlalchemy==2.0.23",
    "psycopg2-binary==2.9.9",
    "pydantic==2.5.0",
    "pydantic-settings==2.1.0",
    "alembic==1.12.1",
    "python-dotenv==1.0.0"
]

[project.optional-dependencies]
dev = [
    "pytest==7.4.3",
    "pytest-asyncio==0.21.1",
    "black==23.12.0",
    "ruff==0.1.8"
]
```

### main.py
```python
from fastapi import FastAPI
from fastapi.middleware.cors import CORSMiddleware
from app.api.routes import router
from app.database import engine, Base

# Create tables
Base.metadata.create_all(bind=engine)

app = FastAPI(
    title="{{ project_name }}",
    description="API for {{ project_name }}",
    version="0.1.0"
)

# CORS middleware
app.add_middleware(
    CORSMiddleware,
    allow_origins=["*"],
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)

# Include routes
app.include_router(router, prefix="/api/v1")

@app.get("/health")
async def health_check():
    return {"status": "healthy"}

if __name__ == "__main__":
    import uvicorn
    uvicorn.run("main:app", host="0.0.0.0", port=8000, reload=True)
```

### models.py
```python
from sqlalchemy import Column, Integer, String, DateTime, Boolean
from sqlalchemy.ext.declarative import declarative_base
from datetime import datetime

Base = declarative_base()

class User(Base):
    __tablename__ = "users"

    id = Column(Integer, primary_key=True, index=True)
    email = Column(String, unique=True, index=True)
    username = Column(String, unique=True, index=True)
    full_name = Column(String)
    is_active = Column(Boolean, default=True)
    created_at = Column(DateTime, default=datetime.utcnow)
    updated_at = Column(DateTime, default=datetime.utcnow, onupdate=datetime.utcnow)
```

### schemas.py
```python
from pydantic import BaseModel, EmailStr
from datetime import datetime
from typing import Optional

class UserBase(BaseModel):
    email: EmailStr
    username: str
    full_name: Optional[str] = None

class UserCreate(UserBase):
    pass

class UserUpdate(BaseModel):
    full_name: Optional[str] = None
    is_active: Optional[bool] = None

class User(UserBase):
    id: int
    is_active: bool
    created_at: datetime
    updated_at: datetime

    class Config:
        from_attributes = True
```

### database.py
```python
from sqlalchemy import create_engine
from sqlalchemy.orm import sessionmaker
from sqlalchemy.ext.declarative import declarative_base
import os

DATABASE_URL = os.getenv(
    "DATABASE_URL",
    "postgresql://user:password@localhost/{{ project_name }}"
)

engine = create_engine(DATABASE_URL)
SessionLocal = sessionmaker(autocommit=False, autoflush=False, bind=engine)
Base = declarative_base()

def get_db():
    db = SessionLocal()
    try:
        yield db
    finally:
        db.close()
```

### app/api/routes.py
```python
from fastapi import APIRouter, Depends, HTTPException
from sqlalchemy.orm import Session
from app.schemas import User, UserCreate, UserUpdate
from app.crud.user import create_user, get_user, list_users, update_user
from app.database import get_db

router = APIRouter()

@router.post("/users", response_model=User)
async def create_new_user(user: UserCreate, db: Session = Depends(get_db)):
    return create_user(db, user)

@router.get("/users/{user_id}", response_model=User)
async def get_user_by_id(user_id: int, db: Session = Depends(get_db)):
    user = get_user(db, user_id)
    if not user:
        raise HTTPException(status_code=404, detail="User not found")
    return user

@router.get("/users", response_model=list[User])
async def list_all_users(skip: int = 0, limit: int = 10, db: Session = Depends(get_db)):
    return list_users(db, skip=skip, limit=limit)

@router.put("/users/{user_id}", response_model=User)
async def update_user_by_id(user_id: int, user: UserUpdate, db: Session = Depends(get_db)):
    return update_user(db, user_id, user)
```

### app/crud/user.py
```python
from sqlalchemy.orm import Session
from app.models import User
from app.schemas import UserCreate, UserUpdate

def create_user(db: Session, user: UserCreate):
    db_user = User(**user.dict())
    db.add(db_user)
    db.commit()
    db.refresh(db_user)
    return db_user

def get_user(db: Session, user_id: int):
    return db.query(User).filter(User.id == user_id).first()

def list_users(db: Session, skip: int = 0, limit: int = 10):
    return db.query(User).offset(skip).limit(limit).all()

def update_user(db: Session, user_id: int, user: UserUpdate):
    db_user = get_user(db, user_id)
    if not db_user:
        return None
    update_data = user.dict(exclude_unset=True)
    for field, value in update_data.items():
        setattr(db_user, field, value)
    db.add(db_user)
    db.commit()
    db.refresh(db_user)
    return db_user
```

### docker-compose.yml
```yaml
version: '3.8'

services:
  postgres:
    image: postgres:16-alpine
    environment:
      POSTGRES_USER: user
      POSTGRES_PASSWORD: password
      POSTGRES_DB: {{ project_name }}
    ports:
      - "5432:5432"
    volumes:
      - postgres_data:/var/lib/postgresql/data

  api:
    build: .
    command: python main.py
    ports:
      - "8000:8000"
    environment:
      DATABASE_URL: postgresql://user:password@postgres/{{ project_name }}
    depends_on:
      - postgres
    volumes:
      - .:/app

volumes:
  postgres_data:
```

### Dockerfile
```dockerfile
FROM python:3.12-slim

WORKDIR /app

COPY pyproject.toml requirements.txt ./
RUN pip install --no-cache-dir -r requirements.txt

COPY . .

CMD ["python", "main.py"]
```

### requirements.txt
```
fastapi==0.104.1
uvicorn[standard]==0.24.0
sqlalchemy==2.0.23
psycopg2-binary==2.9.9
pydantic==2.5.0
pydantic-settings==2.1.0
alembic==1.12.1
python-dotenv==1.0.0
```

## Getting Started

1. Install dependencies:
   ```bash
   pip install -r requirements.txt
   ```

2. Set up database:
   ```bash
   docker-compose up -d postgres
   alembic upgrade head
   ```

3. Run development server:
   ```bash
   python main.py
   ```

4. Access API documentation:
   - Swagger UI: http://localhost:8000/docs
   - ReDoc: http://localhost:8000/redoc

## Environment Variables

Create a `.env` file:
```
DATABASE_URL=postgresql://user:password@localhost/{{ project_name }}
```
