"""
# GitHub User API Server - FastAPI Implementation

This module provides a REST API server for fetching and managing GitHub user information.
The server is built using FastAPI and provides automatic OpenAPI documentation.

## Features

- Fetch individual GitHub users by username
- Create users with auto-incrementing IDs
- Search users with pagination
- List users with pagination
- Built-in statistics endpoint
- Automatic OpenAPI/Swagger documentation
- Static file serving for documentation

## Usage

```python
import asyncio
from github_user_fetcher.adapter import HttpAdapter
from github_user_fetcher.server import create_app

async def main():
    adapter = HttpAdapter("https://api.github.com")
    app = create_app(adapter)
    # Run with uvicorn: uvicorn github_user_fetcher.server:app --host 0.0.0.0 --port 3000
```
"""

import asyncio
from typing import Optional, List, Dict, Any
from threading import Lock
from pathlib import Path

from fastapi import FastAPI, HTTPException, Depends, Query, Path as PathParam
from fastapi.responses import RedirectResponse, FileResponse
from fastapi.staticfiles import StaticFiles
from fastapi.middleware.cors import CORSMiddleware
from pydantic import BaseModel, Field, validator
from contextlib import asynccontextmanager

from ..adapter import HttpAdapter
from ..user import fetch_github_user

# from . import stats


# Global counter for generating unique user IDs - starts from 1
_user_id_counter = 1
_counter_lock = Lock()


def get_next_user_id() -> int:
    """Generate next unique user ID in thread-safe manner"""
    global _user_id_counter
    with _counter_lock:
        current_id = _user_id_counter
        _user_id_counter += 1
        return current_id


class UserResponse(BaseModel):
    """GitHub user's public profile information"""

    login: str = Field(
        ..., max_length=100, description="GitHub username", example="octocat"
    )
    name: Optional[str] = Field(
        None, max_length=255, description="User's display name", example="The Octocat"
    )
    company: Optional[str] = Field(
        None, max_length=255, description="User's company", example="GitHub"
    )
    location: Optional[str] = Field(
        None, max_length=255, description="User's location", example="San Francisco, CA"
    )

    def with_id(self, user_id: int) -> "UserWithIdResponse":
        """Adds an ID to create a UserWithIdResponse"""
        return UserWithIdResponse(id=user_id, **self.dict())


class UserWithIdResponse(UserResponse):
    """User response with system-generated ID"""

    id: int = Field(..., description="Auto-incremented ID starting from 1", example=42)


class CreateUserRequest(BaseModel):
    """Request to create a new user"""

    username: str = Field(
        ..., description="GitHub username to fetch", example="octocat"
    )


class SearchUsersResponse(BaseModel):
    """Paginated search results for users"""

    total_count: int = Field(..., description="Total matching users", example=1234)
    items: List[UserResponse] = Field(..., description="Current page of user results")


class ListUsersResponse(BaseModel):
    """Paginated user list with cursor"""

    users: List[UserResponse] = Field(..., description="Users in current page")
    since: Optional[int] = Field(
        None, description="Cursor for next page", example=583231
    )


class ErrorResponse(BaseModel):
    """Standard error response"""

    message: str = Field(
        ...,
        description="Error description",
        example="User 'nonexistent' not found: 404 Not Found",
    )


class APIState:
    """Application state container"""

    def __init__(self, github_adapter: HttpAdapter):
        self.github_adapter = github_adapter


@asynccontextmanager
async def lifespan(app: FastAPI):
    """Application lifespan manager"""
    # Startup
    print(f"Starting GitHub User API Server")
    yield
    # Shutdown
    print("Shutting down GitHub User API Server")


def create_app(github_adapter: HttpAdapter, port: int = 3000) -> FastAPI:
    """Create FastAPI application with all routes and middleware"""

    app = FastAPI(
        title="GitHub User API",
        description="A simple API to fetch GitHub user information",
        summary="GitHub User Information API",
        version="1.0.0",
        servers=[
            {
                "url": f"http://localhost:{port}/api",
                "description": "Local development server",
            }
        ],
        lifespan=lifespan,
    )

    # Add CORS middleware
    app.add_middleware(
        CORSMiddleware,
        allow_origins=["*"],
        allow_credentials=True,
        allow_methods=["*"],
        allow_headers=["*"],
    )

    # Store adapter in app state
    app.state.api = APIState(github_adapter)  # NOTE: what does it store?!

    # Dependency to get GitHub adapter
    def get_github_adapter() -> HttpAdapter:
        return app.state.api.github_adapter

    # Routes
    @app.get("/hello")
    async def hello():
        """Simple hello endpoint"""
        return "Hello, world!"

    @app.get("/{username}", response_model=UserResponse, tags=["Legacy"])
    async def fetch_user_legacy(
        username: str = PathParam(..., description="GitHub username"),
        adapter: HttpAdapter = Depends(get_github_adapter),
    ):
        """Legacy handler for non-OpenAPI user fetch route"""
        try:
            user = await fetch_github_user(adapter, username)
            return UserResponse(
                login=user.login,
                name=user.name,
                company=user.company,
                location=user.location,
            )
        except Exception as e:
            raise HTTPException(
                status_code=404, detail=f"User '{username}' not found: {str(e)}"
            )

    @app.get("/api/{username}", response_model=UserResponse, tags=["Users"])
    async def fetch_user(
        username: str = PathParam(
            ...,
            description="The GitHub username to fetch profile for (e.g., 'octocat', 'torvalds')",
        ),
        adapter: HttpAdapter = Depends(get_github_adapter),
    ):
        """Fetch a GitHub user by username"""
        print(f"username is {username}")
        try:
            user = await fetch_github_user(adapter, username)
            return UserResponse(
                login=user.login,
                name=user.name,
                company=user.company,
                location=user.location,
            )
        except Exception as e:
            raise HTTPException(
                status_code=404, detail=f"User '{username}' not found: {str(e)}"
            )

    @app.post(
        "/api/users", response_model=UserWithIdResponse, status_code=201, tags=["Users"]
    )
    async def create_user(
        request: CreateUserRequest, adapter: HttpAdapter = Depends(get_github_adapter)
    ):
        """Create a new user with auto-generated ID"""
        try:
            user = await fetch_github_user(adapter, request.username)
            user_id = get_next_user_id()

            user_response = UserResponse(
                login=user.login,
                name=user.name,
                company=user.company,
                location=user.location,
            )
            return user_response.with_id(user_id)
        except Exception as e:
            raise HTTPException(
                status_code=404, detail=f"User '{request.username}' not found: {str(e)}"
            )

    @app.get("/api/users/search", response_model=SearchUsersResponse, tags=["Users"])
    async def search_users(
        q: str = Query(
            ...,
            description="Search query string to match against usernames (e.g., 'octo', 'john')",
        ),
        per_page: Optional[int] = Query(
            30, le=100, description="Number of results per page, maximum 100"
        ),
        page: Optional[int] = Query(
            1, ge=1, description="Page number for pagination, starts at 1"
        ),
        adapter: HttpAdapter = Depends(get_github_adapter),
    ):
        """Search GitHub users with pagination"""
        try:
            params = {
                "q": q,
                "type": "user",
                "per_page": min(per_page, 100),
                "page": page,
            }

            response = await adapter.get("/search/users", params=params)
            response.raise_for_status()

            data = response.json()
            total_count = data.get("total_count", 0)
            items = [
                UserResponse(**item)
                for item in data.get("items", [])
                if all(key in item for key in ["login"])
            ]

            return SearchUsersResponse(total_count=total_count, items=items)

        except Exception as e:
            raise HTTPException(status_code=400, detail=f"Search failed: {str(e)}")

    @app.get("/api/users", response_model=ListUsersResponse, tags=["Users"])
    async def list_users(
        since: Optional[int] = Query(
            None,
            description="Cursor for pagination, use the user ID from where to start listing",
        ),
        per_page: Optional[int] = Query(
            30, le=100, description="Number of users per page, maximum 100"
        ),
        adapter: HttpAdapter = Depends(get_github_adapter),
    ):
        """List GitHub users with cursor-based pagination"""
        try:
            params = {"per_page": min(per_page, 100)}
            if since is not None:
                params["since"] = since

            response = await adapter.get("/users", params=params)
            response.raise_for_status()

            users_data = response.json()
            users = [
                UserResponse(**user_data)
                for user_data in users_data
                if all(key in user_data for key in ["login"])
            ]

            next_since = users_data[-1].get("id") if users_data else None

            return ListUsersResponse(users=users, since=next_since)

        except Exception as e:
            raise HTTPException(
                status_code=400, detail=f"Failed to list users: {str(e)}"
            )

    # @app.get("/api/stats", response_model=dict, tags=["Statistics"])
    # async def get_stats():
    #     """Get API statistics and health information"""
    #     return stats.get_stats()

    # Static file serving and documentation redirects
    @app.get("/docs")
    async def docs_redirect():
        """Redirect to Cargo documentation"""
        return RedirectResponse(url="/docs/target/github_user_fetcher/index.html")

    # Mount static files if docs directory exists
    docs_path = Path("docs/target")
    if docs_path.exists():
        app.mount("/docs/target", StaticFiles(directory=str(docs_path)), name="docs")

    return app


def app(port: int = 3000) -> FastAPI:
    """Create server instance - adapter will be injected at runtime"""
    return create_app(HttpAdapter("https://api.github.com"), port)


__all__ = ["create_app", "app"]
