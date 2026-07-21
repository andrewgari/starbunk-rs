import type { NextConfig } from "next";

const nextConfig: NextConfig = {
  output: "standalone",
  async rewrites() {
    return [
      {
        source: "/api/bots/:path*",
        destination: "http://127.0.0.1:9082/api/bots/:path*",
      },
      {
        source: "/api/bots",
        destination: "http://127.0.0.1:9082/api/bots",
      },
    ];
  },
};

export default nextConfig;
