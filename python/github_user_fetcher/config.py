"""
Configuration management for the GitHub User Fetcher CLI tool.

This module provides a Config class for managing runtime configuration
parsed from command-line arguments using argparse.

Depending on the provided arguments, the application runs in either
server mode or CLI mode.

Examples:

    python github_user_fetcher/cli.py --user octocat
    python github_user_fetcher/cli.py --server --port 8080
"""

import argparse
from dataclasses import dataclass
from enum import Enum
from typing import List

DEFAULT_USERNAME = "izelnakri"
DEFAULT_PORT = 1234


class Mode(Enum):
    SERVER = "server"
    CLI = "cli"


@dataclass
class Config:
    mode: Mode
    port: int
    username: str

    @staticmethod
    def from_args(args: List[str]) -> "Config":
        parser = argparse.ArgumentParser(
            description="Fetch GitHub user info or run an HTTP server."
        )

        parser.add_argument("--server", action="store_true", help="Run as HTTP server")
        parser.add_argument(
            "--port", type=int, default=DEFAULT_PORT, help="Port for HTTP server"
        )
        parser.add_argument(
            "--user",
            type=str,
            default=DEFAULT_USERNAME,
            help="GitHub username to fetch",
        )

        parsed = parser.parse_args(args)
        if parsed.server:
            return Config(mode=Mode.SERVER, port=parsed.port, username="")
        else:
            return Config(mode=Mode.CLI, port=0, username=parsed.user)
