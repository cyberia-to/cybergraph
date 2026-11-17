#!/usr/bin/env python3
"""Serve a trunk dist/ under a sub-path, falling back to index.html.

    python3 scripts/serve.py dist --base /pr/studio/ --port 8791

Requests under the base map to files in the directory; a path with no file
behind it gets index.html, so a reload on a client-side route works. Requests
outside the base answer 404, which is how a wrong base shows up in a check.
"""
import argparse
import http.server
import os
import posixpath
import urllib.parse


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("directory")
    ap.add_argument("--base", default="/")
    ap.add_argument("--port", type=int, default=8791)
    args = ap.parse_args()
    root = os.path.abspath(args.directory)
    base = "/" + args.base.strip("/")
    base = base if base.endswith("/") else base + "/"

    class Handler(http.server.SimpleHTTPRequestHandler):
        def translate_path(self, path):
            path = urllib.parse.unquote(urllib.parse.urlsplit(path).path)
            if not (path + "/").startswith(base):
                return os.path.join(root, ".outside-base")
            rel = posixpath.normpath("/" + path[len(base):]).lstrip("/")
            full = os.path.join(root, rel)
            if rel and os.path.isfile(full):
                return full
            return os.path.join(root, "index.html")

        def end_headers(self):
            self.send_header("Cache-Control", "no-store")
            super().end_headers()

        def log_message(self, *a):
            pass

    with http.server.ThreadingHTTPServer(("127.0.0.1", args.port), Handler) as srv:
        print(f"serving {root} at http://127.0.0.1:{args.port}{base}", flush=True)
        srv.serve_forever()


if __name__ == "__main__":
    main()
