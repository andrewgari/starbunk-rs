import NextAuth from "next-auth"
import DiscordProvider from "next-auth/providers/discord"

export const { handlers, auth, signIn, signOut } = NextAuth({
  providers: [
    DiscordProvider({
      clientId: process.env.DISCORD_CLIENT_ID,
      clientSecret: process.env.DISCORD_CLIENT_SECRET,
    }),
  ],
  callbacks: {
    async signIn({ user, account, profile }) {
      const allowedIds = process.env.ADMIN_DISCORD_IDS?.split(",") || [];
      if (user.id && allowedIds.includes(user.id)) {
        return true;
      }
      if (profile?.id && allowedIds.includes(profile.id as string)) {
        return true;
      }
      return false; // Unauthorized
    },
    async session({ session, token }) {
      if (session?.user && token.sub) {
        session.user.id = token.sub;
      }
      return session;
    },
  },
})
