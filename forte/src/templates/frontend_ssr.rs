// Node.js SSR server templates

pub const NODE_SSR_SERVER: &str = r#"// [Generated] SSR server
import express from 'express';
import { renderToString } from 'react-dom/server';
import React from 'react';
import fs from 'fs';
import path from 'path';

const app = express();
const PORT = process.env.PORT || 3000;
const RUST_PORT = process.env.RUST_PORT || 8080;

// Serve static files
app.use('/assets', express.static(path.join(__dirname, '../client/assets')));

// SSR handler
app.get('*', async (req, res) => {
  try {
    // Fetch data from Rust backend
    const backendUrl = `http://localhost:${RUST_PORT}${req.path}`;
    const response = await fetch(backendUrl);

    if (!response.ok) {
      return res.status(response.status).send('Error fetching data');
    }

    const pageProps = await response.json();

    // Dynamically import the page component
    const pagePath = req.path === '/' ? '/index' : req.path;
    const pageModule = await import(`../app${pagePath}/page.js`);
    const Page = pageModule.default;

    // Render to string
    const html = renderToString(React.createElement(Page, pageProps));

    // Send HTML
    res.send(`
      <!DOCTYPE html>
      <html>
        <head>
          <meta charset="UTF-8" />
          <meta name="viewport" content="width=device-width, initial-scale=1.0" />
          <title>Forte App</title>
        </head>
        <body>
          <div id="root">${html}</div>
          <script type="module" src="/assets/client.js"></script>
        </body>
      </html>
    `);
  } catch (err) {
    console.error('SSR Error:', err);
    res.status(500).send('Internal Server Error');
  }
});

app.listen(PORT, () => {
  console.log(`Frontend server listening on http://localhost:${PORT}`);
});
"#;

pub const VITE_SSR_CONFIG: &str = r#"import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'
import path from 'path'

export default defineConfig({
  plugins: [react()],
  resolve: {
    alias: {
      '@': path.resolve(__dirname, './src'),
    },
  },
  build: {
    ssr: true,
    outDir: '../.generated/frontend',
    rollupOptions: {
      input: './src/app/index/page.tsx',
    },
  },
})
"#;

pub const FRONTEND_ENTRY_CLIENT: &str = r#"// [Generated] Client-side hydration
import React from 'react';
import ReactDOM from 'react-dom/client';

// This will be dynamically replaced based on the route
const root = document.getElementById('root');
if (root) {
  ReactDOM.hydrateRoot(root, React.createElement('div', null, root.innerHTML));
}
"#;
