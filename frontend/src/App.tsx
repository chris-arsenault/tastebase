import "./App.css";
import { useCallback, useMemo, useState } from "react";
import { useAuth } from "./hooks/useAuth";
import { useTastings } from "./hooks/useTastings";
import { useFilters } from "./hooks/useFilters";
import { useRecipes } from "./hooks/useRecipes";
import { Header } from "./components/Header";
import { SearchBar } from "./components/SearchBar";
import { TastingCard } from "./components/TastingCard";
import { TastingForm } from "./components/TastingForm";
import { ViewModal } from "./components/ViewModal";
import { DeleteModal } from "./components/DeleteModal";
import { RecipeList } from "./components/RecipeList";
import { RecipeDetail } from "./components/RecipeDetail";
import type { Recipe } from "./types";

type AppSection = "tastings" | "recipes";

const searchPlaceholders: Record<string, string> = {
  drink: "Search drinks...",
  all: "Search...",
  sauce: "Search sauces..."
};

const itemLabels: Record<string, string> = {
  drink: "drink",
  all: "item",
  sauce: "sauce"
};

const themeClass: Record<string, string> = {
  drink: "theme-drink",
  sauce: "theme-sauce",
  all: "theme-sauce"
};

function ContentArea({ tastings, filteredTastings, itemLabel, auth, filters }: Readonly<{
  tastings: ReturnType<typeof useTastings>;
  filteredTastings: ReturnType<typeof useFilters>["filteredTastings"];
  itemLabel: string;
  auth: ReturnType<typeof useAuth>["auth"];
  filters: ReturnType<typeof useFilters>["filters"];
}>) {
  if (tastings.loading) {
    return <div className="loading">Loading your collection...</div>;
  }
  if (filteredTastings.length === 0) {
    const message = tastings.tastings.length === 0
      ? `No ${itemLabel}s yet. Add your first tasting!`
      : `No ${itemLabel}s match your filters.`;
    return (
      <div className="empty-state">
        <span className="empty-icon">{"\uD83C\uDF36\uFE0F"}</span>
        <p>{message}</p>
      </div>
    );
  }
  return (
    <div className="card-grid">
      {/* eslint-disable react-perf/jsx-no-new-function-as-prop -- closures in .map() are unavoidable without coupling TastingCard to parent API */}
      {filteredTastings.map((item) => (
        <TastingCard
          key={item.id}
          item={item}
          auth={auth}
          productTypeFilter={filters.productType}
          rerunId={tastings.rerunId}
          onView={() => tastings.openViewModal(item)}
          onEdit={() => tastings.openEditForm(item)}
          onRerun={() => tastings.handleRerun(item)}
          onDelete={() => tastings.openDeleteModal(item)}
        />
      ))}
      {/* eslint-enable react-perf/jsx-no-new-function-as-prop */}
    </div>
  );
}

function TastingsSection({ tastings, filters, setFilters, filteredTastings, activeFilterCount, resetFilters, auth }: Readonly<{
  tastings: ReturnType<typeof useTastings>;
  filters: ReturnType<typeof useFilters>["filters"];
  setFilters: ReturnType<typeof useFilters>["setFilters"];
  filteredTastings: ReturnType<typeof useFilters>["filteredTastings"];
  activeFilterCount: number;
  resetFilters: () => void;
  auth: ReturnType<typeof useAuth>["auth"];
}>) {
  const searchPlaceholder = searchPlaceholders[filters.productType] ?? "Search...";
  const itemLabel = itemLabels[filters.productType] ?? "item";
  const manualFields = useMemo(() => ({ value: tastings.showManualFields, set: tastings.setShowManualFields }), [tastings.showManualFields, tastings.setShowManualFields]);
  const mediaExpanded = useMemo(() => ({ value: tastings.mediaExpanded, set: tastings.setMediaExpanded }), [tastings.mediaExpanded, tastings.setMediaExpanded]);

  return (
    <>
      <SearchBar
        filters={filters}
        setFilters={setFilters}
        activeFilterCount={activeFilterCount}
        searchPlaceholder={searchPlaceholder}
        onReset={resetFilters}
      />

      {tastings.errorMessage && <div className="error-banner">{tastings.errorMessage}</div>}

      {tastings.formOpen && (
        <TastingForm
          formMode={tastings.formMode}
          form={tastings.form}
          setForm={tastings.setForm}
          manualFields={manualFields}
          mediaExpanded={mediaExpanded}
          submitStatus={tastings.submitStatus}
          viewingRecord={tastings.viewingRecord}
          productType={filters.productType}
          onSubmit={tastings.handleSubmit}
          onClose={tastings.closeForm}
          onError={tastings.setErrorMessage}
        />
      )}

      {tastings.viewOpen && tastings.viewingRecord && (
        <ViewModal record={tastings.viewingRecord} onClose={tastings.closeViewModal} />
      )}

      <main className="content">
        <div className="content-header">
          <span className="content-count">{filteredTastings.length} {filteredTastings.length === 1 ? itemLabel : `${itemLabel}s`}</span>
        </div>
        <ContentArea tastings={tastings} filteredTastings={filteredTastings} itemLabel={itemLabel} auth={auth} filters={filters} />
      </main>

      {tastings.deleteTarget && (
        <DeleteModal
          target={tastings.deleteTarget}
          deleting={tastings.deleteStatus === "deleting"}
          onConfirm={tastings.confirmDelete}
          onClose={tastings.closeDeleteModal}
        />
      )}
    </>
  );
}

function RecipesSection({ recipesHook, onSelect }: Readonly<{
  recipesHook: ReturnType<typeof useRecipes>;
  onSelect: (recipe: Recipe) => void;
}>) {
  return (
    <main className="content">
      <div className="content-header">
        <span className="content-count">{recipesHook.recipes.length} recipe{recipesHook.recipes.length !== 1 ? "s" : ""}</span>
      </div>
      <RecipeList
        recipes={recipesHook.recipes}
        loading={recipesHook.loading}
        error={recipesHook.error}
        onSelect={onSelect}
      />
    </main>
  );
}

const App = () => {
  const { auth, menuOpen, setMenuOpen, handleSignIn, handleSignOut } = useAuth();
  const tastings = useTastings(auth);
  const { filters, setFilters, filteredTastings, activeFilterCount, resetFilters } = useFilters(tastings.tastings);
  const recipesHook = useRecipes();

  const [section, setSection] = useState<AppSection>("tastings");
  const [selectedRecipe, setSelectedRecipe] = useState<Recipe | null>(null);
  const clearSelectedRecipe = useCallback(() => setSelectedRecipe(null), []);
  const menu = useMemo(() => ({ open: menuOpen, setOpen: setMenuOpen }), [menuOpen, setMenuOpen]);

  return (
    <div className={`app ${themeClass[filters.productType] ?? "theme-sauce"}`}>
      <Header
        auth={auth}
        filters={filters}
        setFilters={setFilters}
        section={section}
        onSectionChange={setSection}
        formOpen={tastings.formOpen}
        menu={menu}
        onAdd={tastings.openAddForm}
        onCloseForm={tastings.closeForm}
        onSignIn={handleSignIn}
        onSignOut={handleSignOut}
        onError={tastings.setErrorMessage}
      />

      {section === "tastings" && (
        <TastingsSection
          tastings={tastings}
          filters={filters}
          setFilters={setFilters}
          filteredTastings={filteredTastings}
          activeFilterCount={activeFilterCount}
          resetFilters={resetFilters}
          auth={auth}
        />
      )}

      {section === "recipes" && (
        <RecipesSection recipesHook={recipesHook} onSelect={setSelectedRecipe} />
      )}

      {selectedRecipe && (
        <RecipeDetail
          key={selectedRecipe.id}
          recipeId={selectedRecipe.id}
          onClose={clearSelectedRecipe}
        />
      )}

      <footer className="app-footer">
        <span>Copyright &copy; 2025</span>
        <a href="https://ahara.io" target="_blank" rel="noreferrer">
          <img src="/tsonu-combined.png" alt="tsonu" height="14" />
        </a>
      </footer>
    </div>
  );
};

export default App;
