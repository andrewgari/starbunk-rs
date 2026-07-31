def rep(p, o, n):
    with open(p, "r") as f:
        c = f.read()
    with open(p, "w") as f:
        f.write(c.replace(o, n))

rep("starbunk-ui/src/app/bunkbot/actions.ts", "botConfig?: any", "botConfig?: Record<string, unknown>")
rep("starbunk-ui/src/app/bunkbot/actions.ts", "botConfig: any", "botConfig: Record<string, unknown>")
rep("starbunk-ui/src/app/bunkbot/actions.ts", "bots: any[]", "bots: Record<string, unknown>[]")

rep("starbunk-ui/src/app/bunkbot/page.tsx", "b: any", "b: Record<string, unknown>")

rep("starbunk-ui/src/components/AddBotModal.tsx", "doesn't", "doesn&apos;t")
rep("starbunk-ui/src/components/AddBotModal.tsx", "don't", "don&apos;t")
rep("starbunk-ui/src/components/AddBotModal.tsx", "can't", "can&apos;t")
rep("starbunk-ui/src/components/AddBotModal.tsx", "I'm", "I&apos;m")
rep("starbunk-ui/src/components/AddBotModal.tsx", "isn't", "isn&apos;t")
rep("starbunk-ui/src/components/AddBotModal.tsx", "they're", "they&apos;re")

rep("starbunk-ui/src/components/EditBotModal.tsx", "if (isOpen) {\n      setBotName", "if (isOpen) {\n      // eslint-disable-next-line react-hooks/set-state-in-effect\n      setBotName")
rep("starbunk-ui/src/components/EditBotModal.tsx", "as any", "as string")
rep("starbunk-ui/src/components/EditBotModal.tsx", "doesn't", "doesn&apos;t")
rep("starbunk-ui/src/components/EditBotModal.tsx", "don't", "don&apos;t")
rep("starbunk-ui/src/components/EditBotModal.tsx", "can't", "can&apos;t")
rep("starbunk-ui/src/components/EditBotModal.tsx", "I'm", "I&apos;m")

rep("starbunk-ui/src/components/TriggerEditor.tsx", "doesn't", "doesn&apos;t")
rep("starbunk-ui/src/components/TriggerEditor.tsx", "don't", "don&apos;t")
rep("starbunk-ui/src/components/TriggerEditor.tsx", "can't", "can&apos;t")
