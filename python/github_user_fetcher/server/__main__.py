import sys
import uvicorn

uvicorn.run(
    "github_user_fetcher.server:app",
    host="0.0.0.0",
    port=int(sys.argv[1]) if len(sys.argv) > 1 else 1234,
    factory=True,
)
