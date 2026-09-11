#!/usr/bin/env python3
import http.server
import socketserver
import json
import os
import re
import hashlib
import sys

WORKSPACE_DIR = os.path.abspath(os.path.join(os.path.dirname(__file__), ".."))
TARGET_PACKAGE = os.path.join(WORKSPACE_DIR, "target", "package")
CRATES_IO_CACHE = os.path.expanduser("~/.cargo/registry/index/index.crates.io-1949cf8c6b5b557f/.cache")
CRATES_IO_TARBALLS = os.path.expanduser("~/.cargo/registry/cache/index.crates.io-1949cf8c6b5b557f")

PORT = 0

def get_crate_sha256(crate_name, version="0.1.0"):
    path = os.path.join(TARGET_PACKAGE, f"{crate_name}-{version}.crate")
    if os.path.exists(path):
        with open(path, "rb") as f:
            return hashlib.sha256(f.read()).hexdigest()
    return "0" * 64

class RehearsalHandler(http.server.BaseHTTPRequestHandler):
    def log_message(self, format, *args):
        pass

    def do_GET(self):
        if self.path == "/config.json":
            config = {
                "dl": f"http://127.0.0.1:{PORT}/crates/{{crate}}/{{version}}/download",
                "api": f"http://127.0.0.1:{PORT}"
            }
            body = json.dumps(config).encode("utf-8")
            self.send_response(200)
            self.send_header("Content-Type", "application/json")
            self.send_header("Content-Length", str(len(body)))
            self.end_headers()
            self.wfile.write(body)
            return

        if self.path == "/ma/rk/markstone-core":
            cksum = get_crate_sha256("markstone-core")
            entry = {
                "name": "markstone-core",
                "vers": "0.1.0",
                "deps": [
                    {"name": "comrak", "req": "^0.55.0", "features": [], "optional": False, "default_features": False, "target": None, "kind": "normal"},
                    {"name": "serde", "req": "^1.0", "features": ["derive"], "optional": False, "default_features": True, "target": None, "kind": "normal"},
                    {"name": "serde_json", "req": "^1.0", "features": [], "optional": False, "default_features": True, "target": None, "kind": "normal"}
                ],
                "cksum": cksum,
                "features": {},
                "yanked": False
            }
            body = (json.dumps(entry) + "\n").encode("utf-8")
            self.send_response(200)
            self.send_header("Content-Type", "text/plain")
            self.send_header("Content-Length", str(len(body)))
            self.end_headers()
            self.wfile.write(body)
            return

        if self.path == "/ma/rk/markstone-actos":
            cksum = get_crate_sha256("markstone-actos")
            entry = {
                "name": "markstone-actos",
                "vers": "0.1.0",
                "deps": [
                    {"name": "markstone-core", "req": "^0.1.0", "features": [], "optional": False, "default_features": True, "target": None, "kind": "normal"}
                ],
                "cksum": cksum,
                "features": {},
                "yanked": False
            }
            body = (json.dumps(entry) + "\n").encode("utf-8")
            self.send_response(200)
            self.send_header("Content-Type", "text/plain")
            self.send_header("Content-Length", str(len(body)))
            self.end_headers()
            self.wfile.write(body)
            return

        if self.path.startswith("/crates/"):
            parts = self.path.strip("/").split("/")
            if len(parts) == 4 and parts[3] == "download":
                crate = parts[1]
                version = parts[2]
                local_crate = os.path.join(TARGET_PACKAGE, f"{crate}-{version}.crate")
                if os.path.exists(local_crate):
                    with open(local_crate, "rb") as f:
                        data = f.read()
                    self.send_response(200)
                    self.send_header("Content-Type", "application/x-tar")
                    self.send_header("Content-Length", str(len(data)))
                    self.end_headers()
                    self.wfile.write(data)
                    return

                cached_crate = os.path.join(CRATES_IO_TARBALLS, f"{crate}-{version}.crate")
                if os.path.exists(cached_crate):
                    with open(cached_crate, "rb") as f:
                        data = f.read()
                    self.send_response(200)
                    self.send_header("Content-Type", "application/x-tar")
                    self.send_header("Content-Length", str(len(data)))
                    self.end_headers()
                    self.wfile.write(data)
                    return

        # Serve from local crates.io index cache
        rel_path = self.path.lstrip("/")
        local_cache_file = os.path.join(CRATES_IO_CACHE, rel_path)
        if os.path.exists(local_cache_file):
            with open(local_cache_file, "rb") as f:
                content = f.read()
            # Extract JSON lines
            entries = re.findall(rb"\{.*?\"yanked\":\s*(?:true|false)(?:,\s*\"[^\"]+\":[^}]*)*\}", content)
            if entries:
                body = b"\n".join(entries) + b"\n"
                self.send_response(200)
                self.send_header("Content-Type", "text/plain")
                self.send_header("Content-Length", str(len(body)))
                self.end_headers()
                self.wfile.write(body)
                return

        self.send_response(404)
        self.end_headers()

class ThreadedTCPServer(socketserver.ThreadingMixIn, socketserver.TCPServer):
    allow_reuse_address = True
    daemon_threads = True

if __name__ == "__main__":
    server = ThreadedTCPServer(("127.0.0.1", 0), RehearsalHandler)
    PORT = server.server_address[1]
    port_file = sys.argv[1] if len(sys.argv) > 1 else "/tmp/markstone_rehearsal_port.txt"
    with open(port_file, "w") as f:
        f.write(str(PORT))
    try:
        server.serve_forever()
    finally:
        server.server_close()
