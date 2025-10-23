#!/bin/bash

# Build script for WASM version of CIM Keys

set -e

echo "🔨 Building CIM Keys for WASM..."

# Install wasm-pack if not present
if ! command -v wasm-pack &> /dev/null; then
    echo "📦 Installing wasm-pack..."
    curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh
fi

# Build the WASM package
echo "🏗️ Building WASM package..."
wasm-pack build --target web --out-dir pkg --features gui

# Create a simple web server script
cat > serve.py << 'EOF'
#!/usr/bin/env python3
import http.server
import socketserver
import os

PORT = 8080

class MyHTTPRequestHandler(http.server.SimpleHTTPRequestHandler):
    def end_headers(self):
        self.send_header('Cross-Origin-Embedder-Policy', 'require-corp')
        self.send_header('Cross-Origin-Opener-Policy', 'same-origin')
        super().end_headers()

    def guess_type(self, path):
        mimetype = super().guess_type(path)
        if path.endswith('.wasm'):
            return 'application/wasm'
        return mimetype

os.chdir(os.path.dirname(os.path.abspath(__file__)))

with socketserver.TCPServer(("", PORT), MyHTTPRequestHandler) as httpd:
    print(f"🌐 Server running at http://localhost:{PORT}/")
    print("⚠️  Remember: This should only be used on an air-gapped computer!")
    httpd.serve_forever()
EOF

chmod +x serve.py

echo "✅ Build complete!"
echo ""
echo "To run the application:"
echo "  1. Ensure this computer is air-gapped"
echo "  2. Run: ./serve.py"
echo "  3. Open browser to http://localhost:8080"