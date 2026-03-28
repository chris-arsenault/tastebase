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
  userId: string;
  title: string;
  description: string | null;
  baseServings: number;
  notes: string | null;
  source: RecipeSource;
  sourceMeta: Record<string, unknown> | null;
  coverImageUrl: string | null;
  createdAt: string;
  updatedAt: string;
};

export type RecipeIngredient = {
  id: string;
  recipeId: string;
  widgetId: string;
  name: string;
  amount: number;
  unit: string;
  sortOrder: number;
};

export type RecipeStep = {
  id: string;
  recipeId: string;
  widgetId: string;
  title: string;
  content: string;
  timerSeconds: number | null;
  sortOrder: number;
};

export type RecipeFull = Recipe & {
  ingredients: RecipeIngredient[];
  steps: RecipeStep[];
};
