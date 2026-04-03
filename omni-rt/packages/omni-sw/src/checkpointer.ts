import type {
  BaseCheckpointSaver,
  Checkpoint,
  CheckpointMetadata,
  CheckpointTuple,
  PendingWrite,
} from "@langchain/langgraph-checkpoint";
import initSqlJs from "sql.js";
import type { Database, QueryExecResult } from "sql.js";
import { fs, init as initZenfs } from "./zenfs.js";

const enc = new TextEncoder();
const dec = new TextDecoder();

function serialize(value: unknown): [string, Uint8Array] {
  return ["json", enc.encode(JSON.stringify(value))];
}
function deserialize(_type: string, data: Uint8Array): unknown {
  return JSON.parse(dec.decode(data));
}

const DBS = new Map<string, Database>();

async function getDb(threadId: string): Promise<Database> {
  const existing = DBS.get(threadId);
  if (existing) return existing;

  await initZenfs();

  const SQL = await initSqlJs({ locateFile: () => "/sql-wasm.wasm" });

  const dbPath = `/home/checkpoints/${threadId}.sqlite`;
  let buffer: ArrayBuffer | null = null;
  try {
    const data = await fs.promises.readFile(dbPath);
    buffer = (data as Uint8Array).buffer as ArrayBuffer;
  } catch {
    /* new db */
  }

  const db = buffer ? new SQL.Database(new Uint8Array(buffer)) : new SQL.Database();
  db.run(`
    CREATE TABLE IF NOT EXISTS checkpoints (
      thread_id TEXT NOT NULL,
      checkpoint_ns TEXT NOT NULL DEFAULT '',
      checkpoint_id TEXT NOT NULL,
      parent_checkpoint_id TEXT,
      type TEXT,
      checkpoint BLOB,
      metadata BLOB,
      PRIMARY KEY (thread_id, checkpoint_ns, checkpoint_id)
    );
    CREATE TABLE IF NOT EXISTS writes (
      thread_id TEXT NOT NULL,
      checkpoint_ns TEXT NOT NULL DEFAULT '',
      checkpoint_id TEXT NOT NULL,
      task_id TEXT NOT NULL,
      idx INTEGER NOT NULL,
      channel TEXT NOT NULL,
      type TEXT,
      value BLOB,
      PRIMARY KEY (thread_id, checkpoint_ns, checkpoint_id, task_id, idx)
    );
  `);

  await persistDb(db, dbPath);
  DBS.set(threadId, db);
  return db;
}

async function persistDb(db: Database, dbPath: string): Promise<void> {
  const data = db.export();
  await fs.promises.mkdir("/home/checkpoints", { recursive: true });
  await fs.promises.writeFile(dbPath, data);
}

export class SqlJsSaver implements BaseCheckpointSaver {
  // BaseCheckpointSaver requires a serde property
  serde = { dumpsTyped: serialize, loadsTyped: deserialize };

  async getTuple(config: { configurable?: Record<string, unknown> }): Promise<CheckpointTuple | undefined> {
    const threadId = config.configurable?.thread_id as string;
    const checkpointId = config.configurable?.checkpoint_id as string | undefined;
    const db = await getDb(threadId);

    let rows: QueryExecResult[];
    if (checkpointId) {
      rows = db.exec(
        "SELECT checkpoint_id, parent_checkpoint_id, type, checkpoint, metadata FROM checkpoints WHERE thread_id=? AND checkpoint_ns='' AND checkpoint_id=?",
        [threadId, checkpointId],
      );
    } else {
      rows = db.exec(
        "SELECT checkpoint_id, parent_checkpoint_id, type, checkpoint, metadata FROM checkpoints WHERE thread_id=? AND checkpoint_ns='' ORDER BY checkpoint_id DESC LIMIT 1",
        [threadId],
      );
    }

    if (!rows[0]?.values?.length) return undefined;
    const [cpId, parentId, type, cpBlob, metaBlob] = rows[0].values[0] as [
      string,
      string | null,
      string,
      Uint8Array,
      Uint8Array,
    ];

    const checkpoint = deserialize(type, cpBlob) as Checkpoint;
    const metadata = deserialize("json", metaBlob) as CheckpointMetadata;

    const writeRows = db.exec(
      "SELECT task_id, channel, type, value FROM writes WHERE thread_id=? AND checkpoint_ns='' AND checkpoint_id=?",
      [threadId, cpId],
    );

    const pendingWrites: PendingWrite[] = [];
    for (const row of writeRows[0]?.values ?? []) {
      const [taskId, channel, wType, wBlob] = row as [string, string, string, Uint8Array];
      const value = deserialize(wType, wBlob);
      pendingWrites.push([taskId, channel, value] as unknown as PendingWrite);
    }

    return {
      config: { configurable: { thread_id: threadId, checkpoint_ns: "", checkpoint_id: cpId } },
      checkpoint,
      metadata,
      parentConfig: parentId
        ? { configurable: { thread_id: threadId, checkpoint_ns: "", checkpoint_id: parentId } }
        : undefined,
      pendingWrites,
    };
  }

  async *list(
    config: { configurable?: Record<string, unknown> },
    options?: { limit?: number },
  ): AsyncGenerator<CheckpointTuple> {
    const threadId = config.configurable?.thread_id as string;
    const db = await getDb(threadId);
    let sql =
      "SELECT checkpoint_id, parent_checkpoint_id, type, checkpoint, metadata FROM checkpoints WHERE thread_id=? AND checkpoint_ns='' ORDER BY checkpoint_id DESC";
    const params: unknown[] = [threadId];
    if (options?.limit) {
      sql += " LIMIT ?";
      params.push(options.limit);
    }
    const rows = db.exec(sql, params);
    for (const row of rows[0]?.values ?? []) {
      const [cpId, parentId, type, cpBlob, metaBlob] = row as [string, string | null, string, Uint8Array, Uint8Array];
      const checkpoint = deserialize(type, cpBlob) as Checkpoint;
      const metadata = deserialize("json", metaBlob) as CheckpointMetadata;
      yield {
        config: { configurable: { thread_id: threadId, checkpoint_ns: "", checkpoint_id: cpId } },
        checkpoint,
        metadata,
        parentConfig: parentId
          ? { configurable: { thread_id: threadId, checkpoint_ns: "", checkpoint_id: parentId } }
          : undefined,
      };
    }
  }

  async put(
    config: { configurable?: Record<string, unknown> },
    checkpoint: Checkpoint,
    metadata: CheckpointMetadata,
  ): Promise<{ configurable?: Record<string, unknown> }> {
    const threadId = config.configurable?.thread_id as string;
    const db = await getDb(threadId);
    const checkpointId = checkpoint.id;
    const parentId = config.configurable?.checkpoint_id as string | undefined;

    const [cpType, cpBlob] = serialize(checkpoint);
    const [, metaBlob] = serialize(metadata);

    db.run(
      "INSERT OR REPLACE INTO checkpoints (thread_id, checkpoint_ns, checkpoint_id, parent_checkpoint_id, type, checkpoint, metadata) VALUES (?,?,?,?,?,?,?)",
      [threadId, "", checkpointId, parentId ?? null, cpType, cpBlob, metaBlob],
    );

    await persistDb(db, `/home/checkpoints/${threadId}.sqlite`);
    return { configurable: { thread_id: threadId, checkpoint_ns: "", checkpoint_id: checkpointId } };
  }

  async putWrites(
    config: { configurable?: Record<string, unknown> },
    writes: PendingWrite[],
    taskId: string,
  ): Promise<void> {
    const threadId = config.configurable?.thread_id as string;
    const checkpointId = config.configurable?.checkpoint_id as string;
    const db = await getDb(threadId);

    for (let idx = 0; idx < writes.length; idx++) {
      const [channel, value] = writes[idx] as [string, unknown];
      const [wType, wBlob] = serialize(value);
      db.run(
        "INSERT OR REPLACE INTO writes (thread_id, checkpoint_ns, checkpoint_id, task_id, idx, channel, type, value) VALUES (?,?,?,?,?,?,?,?)",
        [threadId, "", checkpointId, taskId, idx, channel, wType, wBlob],
      );
    }
    await persistDb(db, `/home/checkpoints/${threadId}.sqlite`);
  }
}
