"use server";

import * as fs from "fs/promises";
import * as path from "path";
import { revalidatePath } from "next/cache";

const LOCAL_CONFIG_PATH = path.join(process.cwd(), "..", "config");

const BUNKBOT_API_URL = process.env.BUNKBOT_API_URL || "http://localhost:9082";
const COVABOT_API_URL = process.env.COVABOT_API_URL || "http://localhost:9083";
const DJCOVA_API_URL = process.env.DJCOVA_API_URL || "http://localhost:9084";

const STATUS_TIMEOUT_MS = 2000;

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
    const dirPath = path.join(LOCAL_CONFIG_PATH, botName);
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
    const filePath = path.join(LOCAL_CONFIG_PATH, botName, filename);
    if (content === null) {
      await fs.unlink(filePath).catch(() => {});
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
    const res = await fetch("http://localhost:9083/api/personality", {
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
    const res = await fetch("http://localhost:9083/api/personality", {
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
