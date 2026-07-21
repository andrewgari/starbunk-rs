import { NextResponse } from "next/server";
import type { NextRequest } from "next/server";

export function middleware(request: NextRequest) {
  if (request.nextUrl.pathname.startsWith("/api/bots")) {
    const requestHeaders = new Headers(request.headers);
    const token = process.env.BUNKBOT_ADMIN_TOKEN || "";
    if (token) {
      requestHeaders.set("Authorization", `Bearer ${token}`);
    }

    const bunkbotUrl = process.env.BUNKBOT_API_URL || "http://127.0.0.1:9082";
    const targetUrl = new URL(
      request.nextUrl.pathname + request.nextUrl.search,
      bunkbotUrl
    );

    return NextResponse.rewrite(targetUrl, {
      request: {
        headers: requestHeaders,
      },
    });
  }
}

export const config = {
  matcher: "/api/bots/:path*",
};
