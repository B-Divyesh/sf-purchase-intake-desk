import type { DemoState } from '../domain/model';

export const WORKSPACE_DATABASE = 'intake-desk:workspace:v1';
const STORE = 'workspace-state';
const STATE_KEY = 'active-purchase-order';

function openDatabase(): Promise<IDBDatabase> {
  return new Promise((resolve, reject) => {
    const request = indexedDB.open(WORKSPACE_DATABASE, 1);
    request.onupgradeneeded = () => request.result.createObjectStore(STORE);
    request.onsuccess = () => resolve(request.result);
    request.onerror = () => reject(request.error ?? new Error('The workspace could not open.'));
  });
}

export const workspaceRepository = {
  async load(): Promise<DemoState | null> {
    const database = await openDatabase();
    try {
      return await new Promise((resolve, reject) => {
        const request = database.transaction(STORE, 'readonly').objectStore(STORE).get(STATE_KEY);
        request.onsuccess = () => resolve((request.result as DemoState | undefined) ?? null);
        request.onerror = () => reject(request.error);
      });
    } finally { database.close(); }
  },
  async save(state: DemoState): Promise<void> {
    const database = await openDatabase();
    try {
      await new Promise<void>((resolve, reject) => {
        const transaction = database.transaction(STORE, 'readwrite');
        transaction.objectStore(STORE).put(structuredClone(state), STATE_KEY);
        transaction.oncomplete = () => resolve();
        transaction.onerror = () => reject(transaction.error);
      });
    } finally { database.close(); }
  },
  async clear(): Promise<void> {
    const database = await openDatabase();
    try {
      await new Promise<void>((resolve, reject) => {
        const transaction = database.transaction(STORE, 'readwrite');
        transaction.objectStore(STORE).delete(STATE_KEY);
        transaction.oncomplete = () => resolve();
        transaction.onerror = () => reject(transaction.error);
      });
    } finally { database.close(); }
  },
};
