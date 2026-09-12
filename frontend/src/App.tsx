import "./App.css";
import "./styles/filter-bar.css";
import "./styles/signals.css";
import "./styles/books.css";
import { useCallback, useMemo } from "react";
import { useAuth } from "./hooks/useAuth";
import { useTastings } from "./hooks/useTastings";
import { useFilters } from "./hooks/useFilters";
import { useRecipes } from "./hooks/useRecipes";
import {
  useRecipeDiscovery,
  type RecipeSort,
} from "./hooks/useRecipeDiscovery";
import { useBooks } from "./hooks/useBooks";
import { useAppRouter } from "./hooks/useAppRouter";
import { Header } from "./components/Header";
import { SearchBar } from "./components/SearchBar";
import { FilterBar } from "./components/FilterBar";
import { Legend } from "./components/signals";
import { TastingCard } from "./components/TastingCard";
import { TastingForm } from "./components/TastingForm";
import { ViewModal } from "./components/ViewModal";
import { DeleteModal } from "./components/DeleteModal";
import { RecipeList } from "./components/RecipeList";
import { RecipeDetail } from "./components/RecipeDetail";
import { BooksSection } from "./components/BooksSection";
import type { AppSection, Recipe } from "./types";

const searchPlaceholders: Record<string, string> = {
  drink: "Search drinks...",
  all: "Search...",
  sauce: "Search sauces...",
};

const itemLabels: Record<string, string> = {
  drink: "drink",
  all: "item",
  sauce: "sauce",
};

const themeClass: Record<string, string> = {
  drink: "theme-drink",
  sauce: "theme-sauce",
  all: "theme-sauce",
};

const copyrightYear = new Date().getUTCFullYear();

function ContentArea({
  tastings,
  filteredTastings,
  itemLabel,
  auth,
  filters,
}: Readonly<{
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
    const message =
      tastings.tastings.length === 0
        ? `No ${itemLabel}s yet. Add your first tasting!`
        : `No ${itemLabel}s match your filters.`;
    return (
      <div className="empty-state">
        <span className="empty-icon">{"🌶️"}</span>
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

function TastingFormWrapper({
  tastings,
  filters,
}: Readonly<{
  tastings: ReturnType<typeof useTastings>;
  filters: ReturnType<typeof useFilters>["filters"];
}>) {
  const manualFields = useMemo(
    () => ({
      value: tastings.showManualFields,
      set: tastings.setShowManualFields,
    }),
    [tastings.showManualFields, tastings.setShowManualFields],
  );
  const mediaExpanded = useMemo(
    () => ({ value: tastings.mediaExpanded, set: tastings.setMediaExpanded }),
    [tastings.mediaExpanded, tastings.setMediaExpanded],
  );
  return (
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
  );
}

function TastingsSection({
  tastings,
  filtering,
  auth,
}: Readonly<{
  tastings: ReturnType<typeof useTastings>;
  filtering: ReturnType<typeof useFilters>;
  auth: ReturnType<typeof useAuth>["auth"];
}>) {
  const { filters, setFilters, filteredTastings } = filtering;
  const searchPlaceholder =
    searchPlaceholders[filters.productType] ?? "Search...";
  const itemLabel = itemLabels[filters.productType] ?? "item";

  return (
    <>
      <SearchBar
        filters={filters}
        setFilters={setFilters}
        activeFilters={filtering.activeFilters}
        resultCount={filteredTastings.length}
        itemLabel={itemLabel}
        searchPlaceholder={searchPlaceholder}
        onReset={filtering.resetFilters}
        onClearFilter={filtering.clearFilter}
      />
      {tastings.errorMessage && (
        <div className="error-banner">{tastings.errorMessage}</div>
      )}
      {tastings.formOpen && (
        <TastingFormWrapper tastings={tastings} filters={filters} />
      )}
      {tastings.viewOpen && tastings.viewingRecord && (
        <ViewModal
          record={tastings.viewingRecord}
          onClose={tastings.closeViewModal}
        />
      )}
      <main className="content">
        <ContentArea
          tastings={tastings}
          filteredTastings={filteredTastings}
          itemLabel={itemLabel}
          auth={auth}
          filters={filters}
        />
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

const recipeSortOptions: { value: RecipeSort; label: string }[] = [
  { value: "createdAt", label: "Date" },
  { value: "title", label: "Name" },
  { value: "score", label: "Score" },
];

const recipeLegend = [
  { glyph: <span className="legend-score">8/10</span>, label: "latest score" },
  { glyph: "↗", label: "linked recipe" },
];

const noActiveFilters: never[] = [];
const noop = () => {};

function RecipesSection({
  recipesHook,
  onSelect,
}: Readonly<{
  recipesHook: ReturnType<typeof useRecipes>;
  onSelect: (recipe: Recipe) => void;
}>) {
  const discovery = useRecipeDiscovery(recipesHook.recipes);
  const search = useMemo(
    () => ({
      value: discovery.search,
      placeholder: "Search recipes...",
      onChange: discovery.setSearch,
    }),
    [discovery.search, discovery.setSearch],
  );
  const sort = useMemo(
    () => ({
      options: recipeSortOptions,
      value: discovery.sort,
      direction: discovery.direction,
      onChange: discovery.setSort,
      onToggleDirection: discovery.toggleDirection,
    }),
    [
      discovery.direction,
      discovery.setSort,
      discovery.sort,
      discovery.toggleDirection,
    ],
  );
  const resultCount = useMemo(
    () => ({ count: discovery.visibleRecipes.length, noun: "recipe" }),
    [discovery.visibleRecipes.length],
  );
  const legend = useMemo(() => <Legend items={recipeLegend} />, []);
  return (
    <>
      <FilterBar
        search={search}
        sort={sort}
        resultCount={resultCount}
        activeFilters={noActiveFilters}
        onClearAll={noop}
        legend={legend}
      />
      <main className="content">
        <RecipeList
          recipes={discovery.visibleRecipes}
          loading={recipesHook.loading}
          error={recipesHook.error}
          onSelect={onSelect}
          filtered={discovery.search.trim().length > 0}
        />
      </main>
    </>
  );
}

function AppFooter() {
  return (
    <footer className="app-footer">
      <span>&copy; {copyrightYear} Tastebase</span>
      <a href="https://ahara.io" target="_blank" rel="noreferrer">
        <img src="/tsonu-combined.png" alt="tsonu" height="14" />
      </a>
    </footer>
  );
}

function useDataRefresh(
  section: AppSection,
  tastings: ReturnType<typeof useTastings>,
  recipesHook: ReturnType<typeof useRecipes>,
  booksHook: ReturnType<typeof useBooks>,
) {
  return useMemo(() => {
    if (section === "recipes") {
      return { refreshing: recipesHook.loading, onRefresh: recipesHook.reload };
    }
    if (section === "books") {
      return { refreshing: booksHook.loading, onRefresh: booksHook.reload };
    }
    return { refreshing: tastings.refreshing, onRefresh: tastings.refresh };
  }, [
    section,
    tastings.refreshing,
    tastings.refresh,
    recipesHook.loading,
    recipesHook.reload,
    booksHook.loading,
    booksHook.reload,
  ]);
}

function useMenuState(authHook: ReturnType<typeof useAuth>) {
  return useMemo(
    () => ({ open: authHook.menuOpen, setOpen: authHook.setMenuOpen }),
    [authHook.menuOpen, authHook.setMenuOpen],
  );
}

function useRecipeDeletion(
  recipesHook: ReturnType<typeof useRecipes>,
  handleBackToRecipes: () => void,
) {
  return useCallback(() => {
    recipesHook.reload();
    handleBackToRecipes();
  }, [recipesHook, handleBackToRecipes]);
}

const App = () => {
  const authHook = useAuth();
  const tastings = useTastings(authHook.auth);
  const filtering = useFilters(tastings.tastings);
  const recipesHook = useRecipes();
  const booksHook = useBooks(authHook.auth);
  const {
    section,
    selectedRecipe,
    selectedReviewId,
    setSection,
    handleSelectRecipe,
    handleBackToRecipes,
  } = useAppRouter(recipesHook.recipes);
  const menu = useMenuState(authHook);
  const dataRefresh = useDataRefresh(section, tastings, recipesHook, booksHook);
  const handleRecipeDeleted = useRecipeDeletion(
    recipesHook,
    handleBackToRecipes,
  );

  return (
    <div
      className={`app ${themeClass[filtering.filters.productType] ?? "theme-sauce"}`}
    >
      <Header
        auth={authHook.auth}
        filters={filtering.filters}
        setFilters={filtering.setFilters}
        section={section}
        onSectionChange={setSection}
        formOpen={tastings.formOpen}
        refresh={dataRefresh}
        menu={menu}
        onAdd={tastings.openAddForm}
        onCloseForm={tastings.closeForm}
        authActions={authHook.authActions}
        onError={tastings.setErrorMessage}
      />
      {section === "tastings" && (
        <TastingsSection
          tastings={tastings}
          filtering={filtering}
          auth={authHook.auth}
        />
      )}
      {section === "recipes" && !selectedRecipe && (
        <RecipesSection
          recipesHook={recipesHook}
          onSelect={handleSelectRecipe}
        />
      )}
      {section === "recipes" && selectedRecipe && (
        <RecipeDetail
          key={selectedRecipe.id}
          recipeId={selectedRecipe.id}
          selectedReviewId={selectedReviewId}
          token={authHook.auth.token}
          onClose={handleBackToRecipes}
          onDeleted={handleRecipeDeleted}
        />
      )}
      {section === "books" && <BooksSection booksHook={booksHook} />}
      <AppFooter />
    </div>
  );
};

export default App;
