import NextAuth from "next-auth"
import DiscordProvider from "next-auth/providers/discord"

export const { handlers, auth, signIn, signOut } = NextAuth({
  providers: [
    DiscordProvider({
      clientId: process.env.DISCORD_CLIENT_ID,
      clientSecret: process.env.DISCORD_CLIENT_SECRET,
      authorization: { params: { scope: "identify guilds" } },
    }),
  ],
  callbacks: {
    async jwt({ token, account }) {
      if (account?.access_token) {
        try {
          const res = await fetch("https://discord.com/api/users/@me/guilds", {
            headers: {
              Authorization: `Bearer ${account.access_token}`
            }
          })
          if (res.ok) {
            const guilds = await res.json()
            const adminGuilds = guilds.filter((g: any) => (Number(g.permissions) & 0x8) === 0x8)
            token.admin_guild_ids = adminGuilds.map((g: any) => g.id)
          }
        } catch (e) {
          console.error("Failed to fetch guilds", e)
        }
      }
      return token;
    },
    async session({ session, token }) {
      if (session?.user && token.sub) {
        session.user.id = token.sub;
      }
      if (token.admin_guild_ids) {
        // @ts-ignore
        session.admin_guild_ids = token.admin_guild_ids;
      }
      return session;
    },
  },
})
