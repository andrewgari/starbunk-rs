"use server";

const COVABOT_API_URL = process.env.COVABOT_API_URL || "http://localhost:9083";

export async function listCovaBotProfiles(): Promise<string[]> {
  try {
    const res = await fetch(`${COVABOT_API_URL}/config/profiles`, { cache: "no-store" });
    if (!res.ok) {
      throw new Error(`Failed to list profiles: ${res.statusText}`);
    }
    return await res.json();
  } catch (error: any) {
    console.error("Error listing CovaBot profiles:", error);
    return [];
  }
}

export async function getCovaBotProfile(id: string): Promise<string | null> {
  try {
    const res = await fetch(`${COVABOT_API_URL}/config/profiles/${id}`, { cache: "no-store" });
    if (!res.ok) {
      if (res.status === 404) return ""; // empty config for new profiles
      throw new Error(`Failed to fetch config: ${res.statusText}`);
    }
    return await res.text();
  } catch (error: any) {
    console.error(`Error fetching CovaBot profile ${id}:`, error);
    return null;
  }
}

export async function saveCovaBotProfile(id: string, yaml: string) {
  try {
    const res = await fetch(`${COVABOT_API_URL}/config/profiles/${id}`, {
      method: "POST",
      headers: {
        "Content-Type": "text/plain",
      },
      body: yaml,
    });
    if (!res.ok) {
      const text = await res.text();
      return { success: false, error: text || res.statusText };
    }
    return { success: true };
  } catch (error: any) {
    console.error(`Error saving CovaBot profile ${id}:`, error);
    return { success: false, error: error.message };
  }
}
