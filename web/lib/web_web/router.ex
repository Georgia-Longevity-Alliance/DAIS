defmodule WebWeb.Router do
  use WebWeb, :router

  pipeline :browser do
    plug :accepts, ["html"]
    plug :fetch_session
    plug :fetch_live_flash
    plug :put_root_layout, html: {WebWeb.Layouts, :root}
    plug :protect_from_forgery
    plug :put_secure_browser_headers
  end

  pipeline :api do
    plug :accepts, ["json"]
  end

  # OAuth routes
  scope "/auth", WebWeb do
    pipe_through :browser

    get "/google", AuthController, :request
    get "/google/callback", AuthController, :callback
    get "/logout", AuthController, :logout
  end

  scope "/", WebWeb do
    pipe_through :browser

    get "/", PageController, :home
    live "/passport/new", PassportLive, :index
    live "/dashboard", DashboardLive, :index
    live "/knowledge", KnowledgeLive, :index
    live "/marketplace", MarketplaceLive, :index
  end

  # API — local passport registry
  scope "/api", WebWeb do
    pipe_through :api

    get "/health", ApiController, :health
    get "/passports", ApiController, :list_passports
    get "/passports/:id", ApiController, :get_passport
    post "/passports", ApiController, :register_passport

    # Knowledge Graph API
    get "/knowledge/claims", ApiController, :list_claims
    post "/knowledge/claims", ApiController, :create_claim
    get "/knowledge/claims/:id", ApiController, :get_claim

    # Auditor core API (Auditor Core Harvest 2026-10-16)
    get "/knowledge/records/:id/status-assertions", ApiController, :list_status_assertions
    post "/knowledge/status-assertions", ApiController, :create_status_assertion
    get "/knowledge/stop-cards", ApiController, :list_stop_cards
    post "/knowledge/stop-cards", ApiController, :create_stop_card
    get "/knowledge/dependency-policies", ApiController, :list_dependency_policies
    post "/knowledge/dependency-policies", ApiController, :create_dependency_policy
    post "/knowledge/composition-policies", ApiController, :create_composition_policy

    # Marketplace API
    get "/marketplace/listings", ApiController, :list_listings
    post "/marketplace/listings", ApiController, :create_listing
    post "/marketplace/buy/:id", ApiController, :buy_listing

    # Economy API
    get "/economy/stats", ApiController, :economy_stats
    get "/economy/transactions", ApiController, :list_transactions
  end
end
