import { createDemoSeed, DEMO_DATABASE, STATE_KEY, type DemoState } from '../domain/model';

export interface DemoRepository {
  load(): Promise<DemoState>;
  save(state: DemoState): Promise<void>;
  reset(): Promise<DemoState>;
}

const STORE = 'demo-state';

function openDatabase(): Promise<IDBDatabase> {
  return new Promise((resolve, reject) => {
    const request = indexedDB.open(DEMO_DATABASE, 1);
    request.onupgradeneeded = () => {
      const database = request.result;
      if (!database.objectStoreNames.contains(STORE)) database.createObjectStore(STORE);
    };
    request.onsuccess = () => resolve(request.result);
    request.onerror = () => reject(request.error ?? new Error('The demo store could not open.'));
  });
}

async function readState(database: IDBDatabase): Promise<DemoState | undefined> {
  return new Promise((resolve, reject) => {
    const request = database.transaction(STORE, 'readonly').objectStore(STORE).get(STATE_KEY);
    request.onsuccess = () => resolve(request.result as DemoState | undefined);
    request.onerror = () => reject(request.error ?? new Error('The demo could not be read.'));
  });
}

async function writeState(database: IDBDatabase, state: DemoState): Promise<void> {
  return new Promise((resolve, reject) => {
    const transaction = database.transaction(STORE, 'readwrite');
    transaction.objectStore(STORE).put(structuredClone(state), STATE_KEY);
    transaction.oncomplete = () => resolve();
    transaction.onerror = () => reject(transaction.error ?? new Error('The demo could not be saved.'));
  });
}

export class IndexedDbDemoRepository implements DemoRepository {
  async load(): Promise<DemoState> {
    const database = await openDatabase();
    try {
      const current = await readState(database);
      if (current) return current;
      const seed = createDemoSeed();
      await writeState(database, seed);
      return seed;
    } finally {
      database.close();
    }
  }

  async save(state: DemoState): Promise<void> {
    const database = await openDatabase();
    try {
      await writeState(database, state);
    } finally {
      database.close();
    }
  }

  async reset(): Promise<DemoState> {
    await new Promise<void>((resolve, reject) => {
      const request = indexedDB.deleteDatabase(DEMO_DATABASE);
      request.onsuccess = () => resolve();
      request.onerror = () => reject(request.error ?? new Error('The demo could not be reset.'));
      request.onblocked = () => reject(new Error('Close another Intake Desk tab, then reset again.'));
    });
    return this.load();
  }
}

export const demoRepository = new IndexedDbDemoRepository();
