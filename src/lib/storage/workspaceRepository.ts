import type { DemoState } from '../domain/model';

/** Offline cache for an inbox. Signed-in Dock records are also retained by the API. */
export const WORKSPACE_DATABASE = 'intake-desk:workspace:v1';
const STORE = 'workspace-state';
const LAST_KEY = 'last-purchase-order';
const LEGACY_KEY = 'active-purchase-order';
const poKey = (id: string) => `po:${id}`;

function openDatabase(): Promise<IDBDatabase> {
  return new Promise((resolve, reject) => {
    const request = indexedDB.open(WORKSPACE_DATABASE, 2);
    request.onupgradeneeded = () => {
      if (!request.result.objectStoreNames.contains(STORE)) request.result.createObjectStore(STORE);
    };
    request.onsuccess = () => resolve(request.result);
    request.onerror = () => reject(request.error ?? new Error('The workspace could not open.'));
  });
}

function value<T>(request: IDBRequest<T>): Promise<T> {
  return new Promise((resolve, reject) => {
    request.onsuccess = () => resolve(request.result);
    request.onerror = () => reject(request.error);
  });
}

async function migrateLegacy(database: IDBDatabase): Promise<void> {
  const legacy = await value(database.transaction(STORE, 'readonly').objectStore(STORE).get(LEGACY_KEY)) as DemoState | undefined;
  if (!legacy) return;
  await new Promise<void>((resolve, reject) => {
    const transaction = database.transaction(STORE, 'readwrite');
    const store = transaction.objectStore(STORE);
    store.put(legacy, poKey(legacy.purchaseOrderId));
    store.put(legacy.purchaseOrderId, LAST_KEY);
    store.delete(LEGACY_KEY);
    transaction.oncomplete = () => resolve();
    transaction.onerror = () => reject(transaction.error);
  });
}

export const workspaceRepository = {
  async load(purchaseOrderId?: string): Promise<DemoState | null> {
    const database = await openDatabase();
    try {
      await migrateLegacy(database);
      const store = database.transaction(STORE, 'readonly').objectStore(STORE);
      const id = purchaseOrderId ?? await value(store.get(LAST_KEY)) as string | undefined;
      return id ? ((await value(store.get(poKey(id)))) as DemoState | undefined ?? null) : null;
    } finally { database.close(); }
  },
  async list(): Promise<DemoState[]> {
    const database = await openDatabase();
    try {
      await migrateLegacy(database);
      const values = await value(database.transaction(STORE, 'readonly').objectStore(STORE).getAll()) as Array<DemoState | string>;
      return values.filter((entry): entry is DemoState => typeof entry === 'object' && entry !== null && 'purchaseOrderId' in entry)
        .sort((a, b) => b.receivedAt.localeCompare(a.receivedAt));
    } finally { database.close(); }
  },
  async save(state: DemoState): Promise<void> {
    const database = await openDatabase();
    try {
      await new Promise<void>((resolve, reject) => {
        const transaction = database.transaction(STORE, 'readwrite');
        const store = transaction.objectStore(STORE);
        store.put(structuredClone(state), poKey(state.purchaseOrderId));
        store.put(state.purchaseOrderId, LAST_KEY);
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
        transaction.objectStore(STORE).clear();
        transaction.oncomplete = () => resolve();
        transaction.onerror = () => reject(transaction.error);
      });
    } finally { database.close(); }
  },
};
