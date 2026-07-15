"use server";

export interface AuditRecord {
  id: number;
  bot_name: string;
  trigger_condition: string;
  output_message: string;
  expected: boolean | null;
  created_at: string;
}

let pool: import("pg").Pool | null = null;

async function getPool() {
  if (pool) return pool;
  if (!process.env.DATABASE_URL) return null;
  try {
    const { Pool } = await import("pg");
    pool = new Pool({ connectionString: process.env.DATABASE_URL });
    return pool;
  } catch {
    return null;
  }
}

export async function getHistory(botName?: string, limit = 50): Promise<AuditRecord[]> {
  const db = await getPool();
  if (!db) {
    return [];
  }

  try {
    let query = "SELECT * FROM bot_audit_history";
    const params: (string | number)[] = [];

    if (botName && botName !== "All") {
      query += " WHERE bot_name = $1 ORDER BY created_at DESC LIMIT $2";
      params.push(botName, limit);
    } else {
      query += " ORDER BY created_at DESC LIMIT $1";
      params.push(limit);
    }

    const res = await db.query(query, params);
    return res.rows.map(row => ({
      ...row,
      created_at: row.created_at.toISOString(),
    }));
  } catch (error) {
    console.error("Error fetching audit history:", error);
    return [];
  }
}
