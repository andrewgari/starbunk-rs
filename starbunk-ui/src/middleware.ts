import { auth } from "@/auth"
import { NextResponse } from "next/server"

export default auth((req) => {
  const isProtectedApi = req.nextUrl.pathname.startsWith('/api/bots')
  
  if (!req.auth) {
    if (isProtectedApi) {
      return NextResponse.json({ error: "Unauthorized" }, { status: 401 })
    }
    // We let the client-side UI component handle the unauthenticated state 
    // so it can render the flashy "RESTRICTED ACCESS" login screen in app/bunkbot/page.tsx
  }
  return NextResponse.next()
})

export const config = {
  matcher: ['/((?!api|_next/static|_next/image|favicon.ico).*)', '/api/bots/:path*'],
}
