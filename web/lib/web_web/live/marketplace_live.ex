defmodule WebWeb.MarketplaceLive do
  use WebWeb, :live_view
  alias Web.Repo
  alias Web.Economy.{Listing, Engine}
  alias Web.Accounts.User
  import Ecto.Query

  @impl true
  def mount(_params, session, socket) do
    user = if session["user_id"], do: Repo.get(User, session["user_id"])
    user_email = session["user_email"]

    listings = Repo.all(
      from l in Listing,
        where: l.active == true,
        order_by: [desc: l.updated_at],
        preload: [:user],
        limit: 50
    )

    stats = Engine.stats()

    {:ok,
     socket
     |> assign(:listings, listings)
     |> assign(:user, user)
     |> assign(:user_email, user_email)
     |> assign(:stats, stats)
     |> assign(:form_title, "")
     |> assign(:form_description, "")
     |> assign(:form_price, "10")
     |> assign(:form_content_type, "knowledge_pack")
     |> assign(:page_title, "Knowledge Marketplace")}
  end

  @impl true
  def render(assigns) do
    ~H"""
    <div class="min-h-screen bg-base-200">
      <div class="navbar bg-base-100 shadow-sm px-6">
        <div class="flex-1">
          <h1 class="text-xl font-bold">🏪 Knowledge Marketplace</h1>
        </div>
        <div class="flex-none gap-2">
          <%= if @user do %>
            <div class="stats shadow mr-2">
              <div class="stat py-1 px-3"><div class="stat-title text-xs">Credits</div><div class="stat-value text-sm"><%= Float.round(@user.credits_balance, 1) %></div></div>
            </div>
          <% end %>
          <a href="/knowledge" class="btn btn-ghost btn-sm">Graph</a>
          <a href="/dashboard" class="btn btn-ghost btn-sm">Dashboard</a>
        </div>
      </div>

      <div class="max-w-6xl mx-auto p-4">
        <!-- Stats -->
        <div class="grid grid-cols-4 gap-4 mb-6">
          <div class="stat bg-base-100 rounded-box shadow p-3"><div class="stat-title text-xs">Claims</div><div class="stat-value text-lg"><%= @stats.total_claims %></div></div>
          <div class="stat bg-base-100 rounded-box shadow p-3"><div class="stat-title text-xs">Users</div><div class="stat-value text-lg"><%= @stats.total_users %></div></div>
          <div class="stat bg-base-100 rounded-box shadow p-3"><div class="stat-title text-xs">Credits</div><div class="stat-value text-lg"><%= Float.round(@stats.total_credits_in_circulation, 0) %></div></div>
          <div class="stat bg-base-100 rounded-box shadow p-3"><div class="stat-title text-xs">Sales</div><div class="stat-value text-lg"><%= @stats.total_sales %></div></div>
        </div>

        <!-- Create listing -->
        <div class="card bg-base-100 shadow-md mb-6">
          <div class="card-body p-4">
            <h2 class="card-title text-sm">Sell Knowledge</h2>
            <form phx-submit="create_listing" class="grid grid-cols-1 md:grid-cols-4 gap-2">
              <input type="text" name="title" placeholder="Title" value={@form_title} class="input input-bordered input-sm" />
              <input type="text" name="description" placeholder="Description" value={@form_description} class="input input-bordered input-sm" />
              <input type="number" name="price_credits" placeholder="Price (credits)" value={@form_price} class="input input-bordered input-sm" step="1" min="1" />
              <select name="content_type" class="select select-bordered select-sm">
                <option value="knowledge_pack">Knowledge Pack</option>
                <option value="dataset">Dataset</option>
                <option value="analysis_report">Analysis Report</option>
                <option value="computation_access">Computation Access</option>
                <option value="consultation">Consultation</option>
              </select>
              <button type="submit" class="btn btn-accent btn-sm col-span-4">List for Sale</button>
            </form>
          </div>
        </div>

        <!-- Buy credits -->
        <%= if @user && @user.credits_balance < 10 do %>
          <div class="alert alert-info mb-6">
            <span>Low credits? Buy more:</span>
            <button class="btn btn-sm" phx-click="buy_credits" phx-value-amount="5">$0.05 = 5 credits</button>
            <button class="btn btn-sm" phx-click="buy_credits" phx-value-amount="10">$0.10 = 10 credits</button>
            <button class="btn btn-sm" phx-click="buy_credits" phx-value-amount="100">$1.00 = 100 credits</button>
          </div>
        <% end %>

        <!-- Listings grid -->
        <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
          <%= for listing <- @listings do %>
            <div class="card bg-base-100 shadow-md">
              <div class="card-body p-4">
                <h3 class="card-title text-sm"><%= listing.title %></h3>
                <p class="text-xs opacity-70"><%= String.slice(listing.description || "", 0, 100) %></p>
                <div class="flex items-center gap-2 mt-2">
                  <span class="badge"><%= listing.content_type %></span>
                  <span class="badge badge-accent"><%= listing.price_credits %> credits</span>
                </div>
                <div class="text-xs opacity-50">
                  by <%= listing.user && listing.user.name || "anon" %> |
                  sold <%= listing.sales_count %>×
                </div>
                <div class="card-actions justify-end mt-2">
                  <%= if @user && @user.id != listing.user_id do %>
                    <button class="btn btn-primary btn-sm" phx-click="buy_listing" phx-value-id={listing.id}>
                      Buy (<%= listing.price_credits %> cr)
                    </button>
                  <% end %>
                  <%= if @user && @user.id == listing.user_id do %>
                    <span class="text-xs opacity-50">Your listing</span>
                  <% end %>
                </div>
              </div>
            </div>
          <% end %>
          <%= if @listings == [] do %>
            <div class="col-span-3 text-center py-8 opacity-50">No listings yet. Be the first to sell knowledge!</div>
          <% end %>
        </div>
      </div>
    </div>
    """
  end

  @impl true
  def handle_event("create_listing", params, socket) do
    user = socket.assigns.user
    if user do
      case Repo.insert(Listing.changeset(%Listing{}, %{
        title: params["title"],
        description: params["description"],
        price_credits: String.to_float(params["price_credits"] || "10"),
        content_type: params["content_type"],
        user_id: user.id
      })) do
        {:ok, _listing} ->
          listings = Repo.all(from l in Listing, where: l.active == true, order_by: [desc: l.updated_at], preload: [:user], limit: 50)
          {:noreply, socket |> assign(:listings, listings) |> put_flash(:info, "Listed for sale!")}
        {:error, _} ->
          {:noreply, put_flash(socket, :error, "Failed to create listing")}
      end
    else
      {:noreply, put_flash(socket, :error, "Please log in")}
    end
  end

  @impl true
  def handle_event("buy_listing", %{"id" => id}, socket) do
    buyer = socket.assigns.user
    listing = Repo.get!(Listing, id)

    if buyer do
      case Engine.purchase_listing(buyer, listing) do
        {:ok, _} ->
          user = Repo.get!(User, buyer.id) |> Repo.preload(:listings)
          listings = Repo.all(from l in Listing, where: l.active == true, order_by: [desc: l.updated_at], preload: [:user], limit: 50)
          {:noreply, socket |> assign(:user, user) |> assign(:listings, listings) |> put_flash(:info, "Purchased: #{listing.title}")}
        {:error, :insufficient_credits, needed: needed} ->
          {:noreply, put_flash(socket, :error, "Need #{Float.round(needed, 1)} more credits. Buy credits below.")}
        {:error, _} ->
          {:noreply, put_flash(socket, :error, "Purchase failed")}
      end
    else
      {:noreply, put_flash(socket, :error, "Please log in")}
    end
  end

  @impl true
  def handle_event("buy_credits", %{"amount" => amount_str}, socket) do
    user = socket.assigns.user
    if user do
      amount = String.to_float(amount_str)
      Engine.purchase_credits(user, amount)
      user = Repo.get!(User, user.id)
      {:noreply, socket |> assign(:user, user) |> put_flash(:info, "Purchased #{trunc(amount / 0.01)} credits for $#{amount}!")}
    else
      {:noreply, put_flash(socket, :error, "Please log in")}
    end
  end
end
