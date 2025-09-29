import type { NextConfig } from "next";

const TARGET_SERVER_BASE_URL =
  process.env.SERVER_BASE_URL || "http://localhost:8081";

const nextConfig: NextConfig = {
  /* config options here */
  async rewrites() {
    return [
      {
        source: "/api/:path*",
        destination: `${TARGET_SERVER_BASE_URL}/api/:path*`,
      },
    ];
  },

  turbopack: {
    rules: {
      "*.svg": {
        loaders: ["@svgr/webpack"],
        as: "*.js",
      },
    },
  },
};

export default nextConfig;
