"use server";

import { revalidatePath } from "next/cache";

const DJCOVA_API_URL = process.env.DJCOVA_API_URL || "http://localhost:9084";

export interface Track {
  title: string;
  url: string;
  requester: string;
  duration_secs?: number;
}

export interface GuildState {
  guild_id: number;
  volume: number;
  is_paused: boolean;
  repeat_mode: string;
  current_track: Track | null;
  queue_length: number;
  history_length: number;
}

export interface DjcovaState {
  guilds: GuildState[];
}

export async function getDjcovaState(): Promise<DjcovaState | null> {
  try {
    const res = await fetch(`${DJCOVA_API_URL}/state`, { cache: "no-store" });
    if (!res.ok) {
      throw new Error(`Failed to fetch state: ${res.statusText}`);
    }
    return await res.json();
  } catch (error: any) {
    console.error("Error fetching DJCova state:", error);
    return null;
  }
}

export async function skipTrack(guildId: number) {
  try {
    const res = await fetch(`${DJCOVA_API_URL}/skip/${guildId}`, { method: "POST" });
    revalidatePath("/djcova");
    return { success: res.ok, error: res.ok ? null : res.statusText };
  } catch (error: any) {
    return { success: false, error: error.message };
  }
}

export async function kickBot(guildId: number) {
  try {
    const res = await fetch(`${DJCOVA_API_URL}/kick/${guildId}`, { method: "POST" });
    revalidatePath("/djcova");
    return { success: res.ok, error: res.ok ? null : res.statusText };
  } catch (error: any) {
    return { success: false, error: error.message };
  }
}
