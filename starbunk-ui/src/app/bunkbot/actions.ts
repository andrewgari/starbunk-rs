"use server";

const BUNKBOT_API_URL = process.env.BUNKBOT_API_URL || "http://localhost:9082";

export async function getBunkBotConfig() {
  try {
    const res = await fetch(`${BUNKBOT_API_URL}/config`, { cache: "no-store" });
    if (!res.ok) {
      throw new Error(`Failed to fetch config: ${res.statusText}`);
    }
    return await res.text();
  } catch (error: any) {
    console.error("Error fetching BunkBot config:", error);
    return null;
  }
}

export async function saveBunkBotConfig(yaml: string) {
  try {
    const res = await fetch(`${BUNKBOT_API_URL}/config`, {
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
    console.error("Error saving BunkBot config:", error);
    return { success: false, error: error.message };
  }
}
