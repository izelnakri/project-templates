"""
This module provides functionality to represent and fetch GitHub user information.

It defines a `User` class representing a GitHub user's public profile fields,
and an async function `fetch_github_user()` for retrieving that data.
"""

from dataclasses import dataclass
from typing import Optional
from github_user_fetcher.adapter import HttpAdapter
import dataclasses
import httpx


@dataclass
class User:
    login: str
    name: Optional[str] = None
    company: Optional[str] = None
    location: Optional[str] = None

    def print(self):
        print(f"Login: {self.login}")
        print(f"Name: {self.name or 'N/A'}")
        print(f"Company: {self.company or 'N/A'}")
        print(f"Location: {self.location or 'N/A'}")


async def fetch_github_user(adapter: HttpAdapter, username: str) -> User:
    try:
        response: httpx.Response = await adapter.get(f"/users/{username}")
        user_data = response.json()
        allowed_keys = {f.name for f in dataclasses.fields(User)}
        filtered_data = {k: v for k, v in user_data.items() if k in allowed_keys}
        return User(**filtered_data)
    except httpx.HTTPStatusError as e:
        raise RuntimeError(
            f"Request failed with status: {e.response.status_code}"
        ) from e
    except Exception as e:
        raise RuntimeError("Failed to fetch GitHub user") from e
