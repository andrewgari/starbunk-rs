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
    console.error("Error fetching BunkBot config:", error);
    return null;
  }
}

export async function saveBunkBotConfig(yaml: string) {
  try {
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
      return { success: false, error: text || res.statusText };
    }

    // Only persist to Kubernetes Secret after the API accepted the config
    const saveResult = await updateBotConfig("bunkbot", "bots.yml", yaml);
    if (!saveResult.success) {
      return saveResult;
    }

    return { success: true };
  } catch (error: unknown) {
    console.error("Error saving BunkBot config:", error);
    return { success: false, error: error instanceof Error ? error.message : String(error) };
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
