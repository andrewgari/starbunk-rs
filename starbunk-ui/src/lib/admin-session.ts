// Server-side session gate shared by the /ragtime page and its server actions.
// Mirrors the BunkBot admin check: a Discord-OAuth session whose user is an
// ADMINISTRATOR in at least one guild (session.admin_guild_ids, set in auth.ts).
import "server-only";

import { auth } from "@/auth";

export type AdminGate =
  | { status: "unauthenticated" }
  | { status: "forbidden" }
  | { status: "ok"; userId?: string };

export async function checkAdmin(): Promise<AdminGate> {
  const session = await auth();
  if (!session?.user) return { status: "unauthenticated" };
  const adminGuildIds = (session as { admin_guild_ids?: string[] }).admin_guild_ids ?? [];
  if (adminGuildIds.length === 0) return { status: "forbidden" };
  return { status: "ok", userId: session.user.id };
}
