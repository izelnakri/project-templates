from github_user_fetcher.adapter import DEFAULT_API_BASE_URL, HttpAdapter
from github_user_fetcher.cli import run_server, run_cli, main
from github_user_fetcher.config import Mode, Config
from github_user_fetcher.user import User, fetch_github_user

# import github_user_fetcher.server as server # NOTE: try this

# TODO: Should I need to add server and tests?

__version__ = "0.1.0"

# NOTE: check if this exposes submodules
__all__ = [
    "DEFAULT_API_BASE_URL",
    "HttpAdapter",
    "run_server",
    "run_cli",
    "main",
    "Mode",
    "Config",
    "User",
    "fetch_github_user",
]
