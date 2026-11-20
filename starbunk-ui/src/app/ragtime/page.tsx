import { redirect } from "next/navigation";
import { checkAdmin } from "@/lib/admin-session";
import QuestionEditor from "./QuestionEditor";
import SignOutButton from "./SignOutButton";

export const dynamic = "force-dynamic";

// Server component: gates on the Discord OAuth session before any editor code
// or data is sent. Logged-out users are redirected to Discord sign-in and come
// back here afterwards.
export default async function RagtimePage() {
  const gate = await checkAdmin();

  if (gate.status === "unauthenticated") {
    redirect(`/api/auth/signin/discord?callbackUrl=${encodeURIComponent("/ragtime")}`);
  }

  if (gate.status === "forbidden") {
    return (
      <div className="flex flex-col items-center justify-center h-full gap-6">
        <div className="glass-panel p-10 max-w-md text-center">
          <h1 className="text-2xl font-bold text-red-400 mb-4">Access denied</h1>
          <p className="text-slate-400 mb-6">
            You need ADMINISTRATOR permission in a Starbunk Discord server to edit Ragtime questions.
          </p>
          <SignOutButton />
        </div>
      </div>
    );
  }

  return <QuestionEditor />;
}
