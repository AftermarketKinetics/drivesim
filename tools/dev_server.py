#!/usr/bin/env python3
"""Local development server for the web builds (docs/drivesim.md §2, §8).

    python tools/dev_server.py [PORT]     # default 8417, serves the repo root

Sends the cross-origin isolation headers (COOP/COEP) that browsers require
before they allow SharedArrayBuffer (threads) and full-resolution timers,
and the WASM MIME type. Binds to localhost only; WSL forwards it to the
Windows browser.
"""

import http.server
import os
import sys

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))


class Handler(http.server.SimpleHTTPRequestHandler):
    extensions_map = {
        **http.server.SimpleHTTPRequestHandler.extensions_map,
        ".wasm": "application/wasm",
        ".js": "text/javascript",
    }

    def __init__(self, *args, **kwargs):
        super().__init__(*args, directory=ROOT, **kwargs)

    def end_headers(self):
        self.send_header("Cross-Origin-Opener-Policy", "same-origin")
        self.send_header("Cross-Origin-Embedder-Policy", "require-corp")
        self.send_header("Cache-Control", "no-store")
        super().end_headers()


def main(argv):
    port = int(argv[1]) if len(argv) > 1 else 8417
    server = http.server.ThreadingHTTPServer(("127.0.0.1", port), Handler)
    print("serving %s on http://localhost:%d/" % (ROOT, port))
    server.serve_forever()


if __name__ == "__main__":
    main(sys.argv)
