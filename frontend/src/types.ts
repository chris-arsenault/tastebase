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
  imageKey?: string;
  imageMimeType?: string;
  ingredientsImageKey?: string;
  ingredientsImageMimeType?: string;
  nutritionImageKey?: string;
  nutritionImageMimeType?: string;
  voiceKey?: string;
  voiceMimeType?: string;
};

export type UpdateTastingMediaInput = {
  imageKey?: string;
  ingredientsImageKey?: string;
  nutritionImageKey?: string;
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
  thumbnailUrl?: string | null;
  latestScore?: number | null;
  createdAt: string;
  updatedAt: string;
};

export type RecipeIngredient = {
  id: string;
  recipeId: string;
  widgetId: string;
  name: string;
  shortName: string;
  amount: number;
  unit: string;
  sortOrder: number;
  linkedRecipeId?: string | null;
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

export type RecipeReview = {
  id: string;
  recipeId: string;
  voiceKey: string | null;
  voiceTranscript: string | null;
  notes: string;
  score: number | null;
  status: string;
  processingError: string | null;
  createdAt: string;
  updatedAt: string;
};

export type RecipeImage = {
  id: string;
  recipeId: string;
  imageUrl: string;
  imageKey: string;
  caption: string;
  sortOrder: number;
  createdAt: string;
};

export type RecipeFull = Recipe & {
  ingredients: RecipeIngredient[];
  steps: RecipeStep[];
  reviews: RecipeReview[];
  images: RecipeImage[];
};
