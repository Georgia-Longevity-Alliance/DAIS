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

    # Marketplace API
    get "/marketplace/listings", ApiController, :list_listings
    post "/marketplace/listings", ApiController, :create_listing
    post "/marketplace/buy/:id", ApiController, :buy_listing

    # Economy API
    get "/economy/stats", ApiController, :economy_stats
    get "/economy/transactions", ApiController, :list_transactions
  end
end
