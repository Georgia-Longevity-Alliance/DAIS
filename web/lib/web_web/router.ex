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
  end

  # API — local passport registry
  scope "/api", WebWeb do
    pipe_through :api

    get "/health", ApiController, :health
    get "/passports", ApiController, :list_passports
    get "/passports/:id", ApiController, :get_passport
    post "/passports", ApiController, :register_passport
  end
end
