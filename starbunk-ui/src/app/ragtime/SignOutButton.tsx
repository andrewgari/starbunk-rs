"use client";

import { signOut } from "next-auth/react";

export default function SignOutButton() {
  return (
    <button onClick={() => signOut()} className="btn-secondary w-full">
      Sign out
    </button>
  );
}
