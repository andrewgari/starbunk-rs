"use server";

const BUNKBOT_API_URL = process.env.BUNKBOT_API_URL || "http://localhost:9082";
import * as yaml from "js-yaml";
import { updateBotConfig, setBotState } from "@/app/actions";

export async function getBunkBotConfig() {
  try {
    const res = await fetch(`${BUNKBOT_API_URL}/config`, { cache: "no-store" });
    if (!res.ok) {
      throw new Error(`Failed to fetch config: ${res.statusText}`);
    }
    return await res.text();
  } catch (error: unknown) {
    console.warn("Error fetching BunkBot config from backend:", error);
    return null;
  }
}

export async function getBunkBotConfigJson() {
  const yamlStr = await getBunkBotConfig();
  if (!yamlStr) return [];
  try {
    const parsed = yaml.load(yamlStr) as { "reply-bots"?: any[] };
    return parsed?.["reply-bots"] || [];
  } catch (e) {
    console.error("Failed to parse fallback yaml", e);
    return [];
  }
}

export async function saveBunkBotConfig(yaml: string) {
  try {
    // 1. Always persist to Kubernetes Secret / Disk first
    const saveResult = await updateBotConfig("bunkbot", "bots.yml", yaml);
    if (!saveResult.success) {
      return saveResult;
    }

    // 2. Try to hot-reload the running API
    const token = process.env.BUNKBOT_ADMIN_TOKEN || "";
    const res = await fetch(`${BUNKBOT_API_URL}/config`, {
      method: "POST",
      headers: {
        "Content-Type": "text/plain",
        ...(token ? { "Authorization": `Bearer ${token}` } : {})
      },
      body: yaml,
    });

    if (!res.ok) {
      const text = await res.text();
      console.warn(`Saved to disk, but hot-reload failed: ${text || res.statusText}`);
      return { success: true, error: `Saved to disk, but hot-reload failed: ${text || res.statusText}` };
    }

    return { success: true };
  } catch (error: unknown) {
    console.warn("Error triggering BunkBot hot-reload API:", error);
    // Return success: true because the disk save succeeded, even though hot-reload failed
    return { success: true, error: "Saved to disk, but backend is offline." };
  }
}

export async function saveBunkBotConfigJson(bots: any[]) {
  try {
    const yamlStr = yaml.dump({ "reply-bots": bots });
    
    return await saveBunkBotConfig(yamlStr);
  } catch (error: any) {
    console.error("Failed to parse and save JSON config", error);
    return { success: false, error: error.message };
  }
}
