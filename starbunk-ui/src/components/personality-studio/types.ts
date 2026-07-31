export interface UserRelationship {
  userId: string;
  alias: string;
  stance: string;
}

export interface NonUserRelationship {
  entity: string;
  stance: string;
}

export interface WeightedPreference {
  item: string;
  weight: number; // -10 to +10
}
