import { http } from '@/lib/http';
import { getToken } from '@/lib/cookie';
import { encrypt } from '@/lib/encrypt';
import { getBaseUrl } from '@/lib/service';
import { selectCredentialMode, requireRustCredentialTransport } from '@/lib/credential-policy';

export async function login(username: string, password: string) {
  const mode = selectCredentialMode(import.meta.env.VITE_AUTH_BACKEND);
  if (mode === 'rust') {
    requireRustCredentialTransport(
      window.location.href,
      getBaseUrl('http'),
      import.meta.env.DEV && import.meta.env.VITE_ALLOW_LOOPBACK_CREDENTIALS === 'true'
    );
  }
  const data = {
    username,
    password: mode === 'rust' ? password : encrypt(password)
  };
  return http.post('/api/auth/login', data);
}

export function logout() {
  return http.post('/api/auth/logout', { token: getToken() });
}

export function getAccount() {
  return http.get('/api/auth/account');
}

export async function changePassword(username: string, password: string, oldPassword?: string) {
  const mode = selectCredentialMode(import.meta.env.VITE_AUTH_BACKEND);
  if (mode === 'rust') {
    requireRustCredentialTransport(
      window.location.href,
      getBaseUrl('http'),
      import.meta.env.DEV && import.meta.env.VITE_ALLOW_LOOPBACK_CREDENTIALS === 'true'
    );
    if (!oldPassword) throw new Error('Current password is required');
    return http.post('/api/auth/password', {
      username,
      old_password: oldPassword,
      new_password: password
    });
  }
  return http.post('/api/auth/password', { username, password: encrypt(password) });
}

export function isPasswordUpdated() {
  return http.get('/api/auth/password');
}
