"""
HTTP adapter with configurable base URL/hostname.

This allows easy testing by pointing to a mock server
instead of real external APIs.
"""

import httpx
from dataclasses import dataclass
from typing import Optional

DEFAULT_API_BASE_URL = "https://api.github.com"


@dataclass
class HttpAdapter:
    base_url: str = DEFAULT_API_BASE_URL
    _client: Optional[httpx.AsyncClient] = None

    def __post_init__(self):
        if self._client is None:
            self._client = httpx.AsyncClient(
                base_url=self.base_url,
                headers={"User-Agent": "github_user_fetcher HttpAdapter"},
                timeout=10.0,
            )

    async def get(self, path: str, **kwargs) -> httpx.Response:
        """
        Perform an async GET request to the specified path.
        """
        url = f"{self.base_url}{path}"
        response = await self._client.get(url, **kwargs)
        response.raise_for_status()
        return response

    async def close(self):
        """
        Gracefully close the underlying HTTP connection.
        """
        if self._client:
            await self._client.aclose()
