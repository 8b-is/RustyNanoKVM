// Explicit backend choice preserves the legacy Go wire format by default.
export function selectCredentialMode(mode: string = 'go') {
  if (mode !== 'go' && mode !== 'rust') throw new Error('Unsupported authentication backend');
  return mode;
}
export function requireRustCredentialTransport(pageAddress: string, apiAddress: string, allowLoopback = false) {
  const page = new URL(pageAddress);
  const api = new URL(apiAddress);
  if (page.username || page.password || api.username || api.password) {
    throw new Error('Credentials must not appear in URLs');
  }
  if (page.protocol === 'https:' && api.protocol === 'https:') return;
  const loopback = new Set(['localhost', '127.0.0.1', '[::1]']);
  if (allowLoopback && page.protocol === 'http:' && api.protocol === 'http:' &&
      loopback.has(page.hostname) && loopback.has(api.hostname)) return;
  throw new Error('Rust credentials require HTTPS or explicit loopback development');
}
