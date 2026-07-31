"use client";

import { useState } from "react";
import { UserRelationship, NonUserRelationship, WeightedPreference } from "./personality-studio/types";
import CoreIdentityPanel from "./personality-studio/CoreIdentityPanel";
import AffinitiesPanel from "./personality-studio/AffinitiesPanel";
import RelationshipsPanel from "./personality-studio/RelationshipsPanel";
import AdvancedSettingsPanel from "./personality-studio/AdvancedSettingsPanel";

export default function PersonalityStudio() {
  // Model Tier States
  const [highTierProvider, setHighTierProvider] = useState("anthropic");
  const [highTierModel, setHighTierModel] = useState("claude-3-5-sonnet-latest");
  const [medTierProvider, setMedTierProvider] = useState("google");
  const [medTierModel, setMedTierModel] = useState("gemini-1.5-flash");
  const [lowTierProvider, setLowTierProvider] = useState("openai");
  const [lowTierModel, setLowTierModel] = useState("text-embedding-3-small");

  // Social Battery Sliders
  const [batteryMax, setBatteryMax] = useState(100);
  const [depletionRate, setDepletionRate] = useState(12);
  const [rechargeRate, setRechargeRate] = useState(5);

  // Core Identity
  const [systemPrompt, setSystemPrompt] = useState(
    "You are Cova, a sharp, cynical Discord user with strong opinions on games and tech. Respond like a real person, not an assistant."
  );
  
  // Manner of Speech & Conversational Style
  const [conversationalStyle, setConversationalStyle] = useState(
    "Dry, witty, often sarcastic but ultimately helpful. Uses analogies related to gaming."
  );
  const [speechPatterns, setSpeechPatterns] = useState(["Casual tone", "Lowercase preference", "No exclamation overload"]);

  // Interests
  const [interests, setInterests] = useState(["Final Fantasy XIV", "Rust Programming", "Mechanical Keyboards"]);

  // Likes and Dislikes (Weighted)
  const [preferences, setPreferences] = useState<WeightedPreference[]>([
    { item: "Clean Code", weight: 8 },
    { item: "Unprocessed Fast Food", weight: -6 },
    { item: "Corporate Jargon", weight: -9 },
  ]);

  // Relationships with Users
  const [userRelationships, setUserRelationships] = useState<UserRelationship[]>([
    { userId: "102938475", alias: "Andrew", stance: "Close Friend & Architect" },
    { userId: "987654321", alias: "Ratbot", stance: "Suspicious Seasonal rival" },
  ]);

  // Relationships with Non-Users
  const [nonUserRelationships, setNonUserRelationships] = useState<NonUserRelationship[]>([
    { entity: "Elon Musk", stance: "Thinks he's a clown" },
    { entity: "JavaScript", stance: "Tolerates it out of necessity" },
  ]);

  return (
    <div className="flex flex-col gap-6">
      <CoreIdentityPanel
        systemPrompt={systemPrompt}
        setSystemPrompt={setSystemPrompt}
        conversationalStyle={conversationalStyle}
        setConversationalStyle={setConversationalStyle}
        speechPatterns={speechPatterns}
        setSpeechPatterns={setSpeechPatterns}
      />

      <AffinitiesPanel
        interests={interests}
        setInterests={setInterests}
        preferences={preferences}
        setPreferences={setPreferences}
      />

      <RelationshipsPanel
        userRelationships={userRelationships}
        setUserRelationships={setUserRelationships}
        nonUserRelationships={nonUserRelationships}
        setNonUserRelationships={setNonUserRelationships}
      />

      <AdvancedSettingsPanel
        highTierProvider={highTierProvider}
        setHighTierProvider={setHighTierProvider}
        highTierModel={highTierModel}
        setHighTierModel={setHighTierModel}
        medTierProvider={medTierProvider}
        setMedTierProvider={setMedTierProvider}
        medTierModel={medTierModel}
        setMedTierModel={setMedTierModel}
        lowTierProvider={lowTierProvider}
        setLowTierProvider={setLowTierProvider}
        lowTierModel={lowTierModel}
        setLowTierModel={setLowTierModel}
        batteryMax={batteryMax}
        setBatteryMax={setBatteryMax}
        depletionRate={depletionRate}
        setDepletionRate={setDepletionRate}
        rechargeRate={rechargeRate}
        setRechargeRate={setRechargeRate}
      />
    </div>
  );
}
