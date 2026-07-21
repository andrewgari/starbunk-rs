import type { NextConfig } from "next";

const bunkbotUrl = process.env.BUNKBOT_API_URL || "http://127.0.0.1:9082";

const nextConfig: NextConfig = {
  output: "standalone",
  async rewrites() {
    return [
      {
        source: "/api/bots/:path*",
        destination: `${bunkbotUrl}/api/bots/:path*`,
      },
      {
        source: "/api/bots",
        destination: `${bunkbotUrl}/api/bots`,
      },
    ];
  },
};

export default nextConfig;
