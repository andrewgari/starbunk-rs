import type { NextConfig } from "next";

const bunkbotUrl = process.env.BUNKBOT_API_URL || "http://127.0.0.1:9082";

const nextConfig: NextConfig = {
  output: "standalone",
};

export default nextConfig;
