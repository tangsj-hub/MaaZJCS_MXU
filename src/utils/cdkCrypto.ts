const BASE_KEY = 'MXU-CDK';

function buildKey(projectName?: string): Uint8Array {
  return new TextEncoder().encode(projectName ? `${BASE_KEY}-${projectName}` : BASE_KEY);
}

export function encryptCdk(plaintext: string, projectName?: string): string {
  if (!plaintext) return '';
  const key = buildKey(projectName);
  const bytes = new TextEncoder().encode(plaintext);
  const xored = new Uint8Array(bytes.length);
  for (let i = 0; i < bytes.length; i++) {
    xored[i] = bytes[i] ^ key[i % key.length];
  }
  return btoa(String.fromCharCode(...xored));
}

export function decryptCdk(encrypted: string, projectName?: string): string {
  if (!encrypted) return '';
  try {
    const key = buildKey(projectName);
    const binary = atob(encrypted);
    const bytes = new Uint8Array(binary.length);
    for (let i = 0; i < binary.length; i++) {
      bytes[i] = binary.charCodeAt(i) ^ key[i % key.length];
    }
    return new TextDecoder().decode(bytes);
  } catch {
    return '';
  }
}
