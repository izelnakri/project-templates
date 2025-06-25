import sys
import os
import asyncio
import uvicorn
from github_user_fetcher.adapter import HttpAdapter, DEFAULT_API_BASE_URL
from github_user_fetcher.config import Config, Mode
from github_user_fetcher.server import create_app
from github_user_fetcher.user import fetch_github_user


def run_server(config: Config):
    print(f"Starting server on port {config.port}")
    uvicorn.run(
        "github_user_fetcher.server:app",
        host="0.0.0.0",
        port=config.port,
        reload=os.getenv("ENVIRONMENT", "development").lower() in ["development"],
        factory=True,
    )


async def run_cli(config: Config):
    github_api_adapter = HttpAdapter(DEFAULT_API_BASE_URL)
    user = await fetch_github_user(github_api_adapter, config.username)
    user.print()


def main():
    config = Config.from_args(sys.argv[1:])
    if config.mode == Mode.SERVER:
        run_server(config)  # <== no await
    elif config.mode == Mode.CLI:
        asyncio.run(run_cli(config))


if __name__ == "__main__":
    main()
