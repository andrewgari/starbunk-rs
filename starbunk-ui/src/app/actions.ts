"use server";

import * as fs from "fs/promises";
import * as path from "path";
import { revalidatePath } from "next/cache";
import { auth } from "@/auth";

const LOCAL_CONFIG_PATH = path.join(process.cwd(), "..", "config");

const BUNKBOT_API_URL = process.env.BUNKBOT_API_URL || "http://localhost:9082";
const COVABOT_API_URL = process.env.COVABOT_API_URL || "http://localhost:3004";
const DJCOVA_API_URL = process.env.DJCOVA_API_URL || "http://localhost:9084";

const STATUS_TIMEOUT_MS = 2000;

const BOT_NAMES = new Set(["bluebot", "bunkbot", "covabot", "djcova", "ratbot"]);
const CONFIG_FILENAME_RE = /^[A-Za-z0-9._-]+\.ya?ml$/;

async function requireSession(): Promise<void> {
  const session = await auth();
  if (!session) throw new Error("Unauthorized");
}

/** Resolve a config path, rejecting unknown bots and any path escaping LOCAL_CONFIG_PATH. */
function resolveConfigPath(botName: string, filename?: string): string {
  if (!BOT_NAMES.has(botName)) throw new Error("Invalid bot name");
  if (filename !== undefined && (filename.includes("..") || !CONFIG_FILENAME_RE.test(filename))) {
    throw new Error("Invalid filename");
  }
  const resolved = path.resolve(LOCAL_CONFIG_PATH, botName, filename ?? "");
  const root = path.resolve(LOCAL_CONFIG_PATH) + path.sep;
  if (!resolved.startsWith(root)) throw new Error("Path escapes config directory");
  return resolved;
}

function errMsg(err: unknown): string {
  return err instanceof Error ? err.message : String(err);
}

export type BotState = "Running" | "Starting" | "Down" | "Unknown";

export interface BotStatus {
  name: string;
  status: BotState;
  /** Human-readable note on how the status was determined. */
  detail: string;
}

/**
 * How each bot's status is probed, using the HTTP API each bot already
 * exposes. bluebot and ratbot only run the shared health server, which binds
 * 127.0.0.1 inside their own container and is unreachable from the UI, so
 * their status is always reported as "Unknown".
 */
const STATUS_PROBES: Record<string, string | null> = {
  bunkbot: `${BUNKBOT_API_URL}/health`,
  covabot: `${COVABOT_API_URL}/api/personality`,
  djcova: `${DJCOVA_API_URL}/state`,
  bluebot: null,
  ratbot: null,
};

async function probe(name: string, url: string | null): Promise<BotStatus> {
  if (!url) {
    return { name, status: "Unknown", detail: "No reachable HTTP endpoint" };
  }
  try {
    const res = await fetch(url, {
      cache: "no-store",
      signal: AbortSignal.timeout(STATUS_TIMEOUT_MS),
    });
    if (res.ok) return { name, status: "Running", detail: `${url} → ${res.status}` };
    if (res.status === 503) return { name, status: "Starting", detail: `${url} → 503` };
    return { name, status: "Down", detail: `${url} → ${res.status}` };
  } catch (err: unknown) {
    return { name, status: "Unknown", detail: `${url} unreachable: ${errMsg(err)}` };
  }
}

export async function getBotStatuses(): Promise<BotStatus[]> {
  return Promise.all(
    Object.entries(STATUS_PROBES).map(([name, url]) => probe(name, url)),
  );
}

export async function getBotConfigs(botName: string): Promise<Record<string, string>> {
  try {
    await requireSession();
    const dirPath = resolveConfigPath(botName);
    await fs.mkdir(dirPath, { recursive: true });
    const files = await fs.readdir(dirPath);
    const configs: Record<string, string> = {};
    for (const file of files) {
      if (file.endsWith(".yml") || file.endsWith(".yaml")) {
        configs[file] = await fs.readFile(path.join(dirPath, file), "utf-8");
      }
    }
    return configs;
  } catch (e) {
    console.error(`Failed to read local configs for ${botName}`, e);
    return {};
  }
}

export async function updateBotConfig(
  botName: string,
  filename: string,
  content: string | null,
): Promise<{ success: boolean; error?: string }> {
  try {
    await requireSession();
    const filePath = resolveConfigPath(botName, filename);
    if (content === null) {
      await fs.unlink(filePath).catch((e: NodeJS.ErrnoException) => {
        if (e.code !== "ENOENT") throw e;
      });
    } else {
      await fs.mkdir(path.dirname(filePath), { recursive: true });
      await fs.writeFile(filePath, content, "utf-8");
    }
    revalidatePath(`/${botName}`);
    return { success: true };
  } catch (e: unknown) {
    return { success: false, error: errMsg(e) };
  }
}

export async function getPersonality() {
  try {
    const res = await fetch(`${COVABOT_API_URL}/api/personality`, {
      cache: "no-store",
    });
    if (!res.ok) throw new Error("Failed to fetch personality");
    return await res.json();
  } catch (error) {
    console.error("Error fetching personality:", error);
    return null;
  }
}

export async function patchPersonality(update: Record<string, unknown>) {
  try {
    await requireSession();
    const res = await fetch(`${COVABOT_API_URL}/api/personality`, {
      method: "PATCH",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(update),
    });
    if (!res.ok) throw new Error("Failed to update personality");
    return { success: true };
  } catch (error) {
    console.error("Error updating personality:", error);
    return { success: false, error: errMsg(error) };
  }
}
