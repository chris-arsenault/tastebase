export type NutritionFacts = {
  servingSize?: string;
  calories?: number;
  totalFat?: string;
  sodium?: string;
  totalCarbs?: string;
  sugars?: string;
  protein?: string;
};

export type ProductType = "sauce" | "drink";

export type TastingRecord = {
  id: string;
  createdAt: string;
  updatedAt: string;
  status?: string;
  processingError?: string;
  productType?: ProductType;
  name: string;
  maker: string;
  date: string;
  score: number | null;
  style: string;
  // Sauce-specific
  heatUser: number | null;
  heatVendor: number | null;
  tastingNotesUser: string;
  tastingNotesVendor: string;
  productUrl: string;
  imageUrl?: string;
  imageKey?: string;
  ingredientsImageUrl?: string;
  ingredientsImageKey?: string;
  nutritionImageUrl?: string;
  nutritionImageKey?: string;
  nutritionFacts?: NutritionFacts;
  ingredients?: string[];
  createdBy?: string;
  needsAttention?: boolean;
  attentionReason?: string;
};

export type CreateTastingInput = {
  name?: string;
  maker?: string;
  date?: string;
  score?: number | null;
  style?: string;
  heatUser?: number | null;
  heatVendor?: number | null;
  tastingNotesUser?: string;
  tastingNotesVendor?: string;
  productUrl?: string;
  imageBase64?: string;
  imageMimeType?: string;
  ingredientsImageBase64?: string;
  ingredientsImageMimeType?: string;
  nutritionImageBase64?: string;
  nutritionImageMimeType?: string;
  voiceBase64?: string;
  voiceMimeType?: string;
};

export type UpdateTastingMediaInput = {
  imageBase64?: string;
  imageMimeType?: string;
  ingredientsImageBase64?: string;
  ingredientsImageMimeType?: string;
  nutritionImageBase64?: string;
  nutritionImageMimeType?: string;
};

export type Filters = {
  productType: ProductType | "all";
  search: string;
  style: string;
  ingredient: string;
  minScore: string;
  minHeat: string;
  date: string;
  sortBy: "date" | "name" | "score" | "style" | "heat";
};

// Recipe types
export type RecipeSource = "claude" | "manual" | "import";

export type Recipe = {
  id: string;
  user_id: string;
  title: string;
  description: string | null;
  base_servings: number;
  notes: string | null;
  source: RecipeSource;
  source_meta: Record<string, unknown> | null;
  cover_image_url: string | null;
  created_at: string;
  updated_at: string;
};

export type RecipeIngredient = {
  id: string;
  recipe_id: string;
  widget_id: string;
  name: string;
  amount: number;
  unit: string;
  sort_order: number;
};

export type RecipeStep = {
  id: string;
  recipe_id: string;
  widget_id: string;
  title: string;
  content: string;
  timer_seconds: number | null;
  sort_order: number;
};

export type RecipeFull = Recipe & {
  ingredients: RecipeIngredient[];
  steps: RecipeStep[];
};
