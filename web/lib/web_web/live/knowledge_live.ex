defmodule WebWeb.KnowledgeLive do
  use WebWeb, :live_view
  alias Web.Repo
  alias Web.Knowledge.Claim
  import Ecto.Query

  @impl true
  def mount(_params, session, socket) do
    claims = Repo.all(from c in Claim, order_by: [desc: c.updated_at], limit: 50, preload: [:user])

    {:ok,
     socket
     |> assign(:claims, claims)
     |> assign(:selected_claim, nil)
     |> assign(:user_email, session["user_email"])
     |> assign(:user_id, session["user_id"])
     |> assign(:form_subject, "")
     |> assign(:form_predicate, "")
     |> assign(:form_object, "")
     |> assign(:form_domain, "")
     |> assign(:page_title, "Knowledge Graph")}
  end

  @impl true
  def render(assigns) do
    ~H"""
    <div class="min-h-screen bg-base-200">
      <div class="navbar bg-base-100 shadow-sm px-6">
        <div class="flex-1">
          <h1 class="text-xl font-bold">🧠 DAIS Knowledge Graph</h1>
        </div>
        <div class="flex-none gap-2">
          <a href="/dashboard" class="btn btn-ghost btn-sm">Dashboard</a>
          <a href="/marketplace" class="btn btn-ghost btn-sm">Marketplace</a>
        </div>
      </div>

      <div class="max-w-6xl mx-auto p-4">
        <!-- Submit form -->
        <div class="card bg-base-100 shadow-md mb-6">
          <div class="card-body p-4">
            <h2 class="card-title text-sm">Submit Knowledge Claim</h2>
            <form phx-submit="submit_claim" class="grid grid-cols-1 md:grid-cols-2 gap-2">
              <input type="text" name="subject" placeholder="Subject (e.g., 'Centriole')" value={@form_subject} class="input input-bordered input-sm" />
              <input type="text" name="predicate" placeholder="Predicate (e.g., 'regulates')" value={@form_predicate} class="input input-bordered input-sm" />
              <input type="text" name="object" placeholder="Object (e.g., 'cell division')" value={@form_object} class="input input-bordered input-sm" />
              <input type="text" name="domain" placeholder="Domain (e.g., 'biology')" value={@form_domain} class="input input-bordered input-sm" />
              <button type="submit" class="btn btn-primary btn-sm col-span-2">Submit Claim (+10 credits for verified)</button>
            </form>
          </div>
        </div>

        <!-- Claims list -->
        <div class="card bg-base-100 shadow-xl">
          <div class="card-body p-0">
            <div class="overflow-x-auto">
              <table class="table table-zebra table-sm">
                <thead><tr><th>Subject</th><th>Predicate</th><th>Object</th><th>Status</th><th>Domain</th><th>By</th></tr></thead>
                <tbody>
                  <%= for c <- @claims do %>
                    <tr class="hover cursor-pointer" phx-click="select_claim" phx-value-id={c.id}>
                      <td class="font-bold"><%= c.subject %></td>
                      <td class="text-primary"><%= c.predicate %></td>
                      <td><%= c.object %></td>
                      <td><span class={"badge badge-xs " <> status_badge(c.status)}><%= c.status %></span></td>
                      <td><span class="badge badge-ghost badge-xs"><%= c.domain %></span></td>
                      <td class="text-xs opacity-70"><%= c.user && c.user.name || c.user && c.user.email || "anon" %></td>
                    </tr>
                  <% end %>
                  <%= if @claims == [] do %>
                    <tr><td colspan="6" class="text-center py-8 opacity-50">No knowledge claims yet. Submit the first one above!</td></tr>
                  <% end %>
                </tbody>
              </table>
            </div>
          </div>
        </div>

        <!-- Selected claim detail -->
        <%= if @selected_claim do %>
          <div class="card bg-base-100 shadow-xl mt-6">
            <div class="card-body">
              <h2 class="card-title">
                <%= @selected_claim.subject %> — <%= @selected_claim.predicate %> — <%= @selected_claim.object %>
                <span class={"badge " <> status_badge(@selected_claim.status)}><%= @selected_claim.status %></span>
              </h2>
              <div class="text-sm opacity-70">
                Domain: <%= @selected_claim.domain %> | Confidence: <%= @selected_claim.confidence %> |
                Cited: <%= @selected_claim.citation_count %>× | Reward: <%= @selected_claim.reward_paid %> credits
              </div>
              <button class="btn btn-ghost btn-sm self-end" phx-click="clear_selection">Close</button>
            </div>
          </div>
        <% end %>
      </div>
    </div>
    """
  end

  @impl true
  def handle_event("submit_claim", params, socket) do
    user_id = socket.assigns.user_id

    if user_id do
      user = Repo.get!(Web.Accounts.User, user_id)

      case Repo.insert(Claim.changeset(%Claim{}, %{
        subject: params["subject"],
        predicate: params["predicate"],
        object: params["object"],
        domain: params["domain"],
        user_id: user.id
      })) do
        {:ok, claim} ->
          # Auto-reward for submission
          Web.Economy.Engine.award_contribution(user, claim, 0.5)

          claims = Repo.all(from c in Claim, order_by: [desc: c.updated_at], limit: 50, preload: [:user])
          {:noreply, socket |> assign(:claims, claims) |> assign(:form_subject, "") |> assign(:form_predicate, "") |> assign(:form_object, "") |> assign(:form_domain, "") |> put_flash(:info, "Claim submitted! +5 credits")}

        {:error, _} ->
          {:noreply, put_flash(socket, :error, "Failed to submit claim")}
      end
    else
      {:noreply, put_flash(socket, :error, "Please log in to submit claims")}
    end
  end

  @impl true
  def handle_event("select_claim", %{"id" => id}, socket) do
    claim = Repo.get!(Claim, id) |> Repo.preload(:user)
    {:noreply, assign(socket, :selected_claim, claim)}
  end

  @impl true
  def handle_event("clear_selection", _, socket) do
    {:noreply, assign(socket, :selected_claim, nil)}
  end

  defp status_badge("proposed"), do: "badge-ghost"
  defp status_badge("supported"), do: "badge-info"
  defp status_badge("tested"), do: "badge-primary"
  defp status_badge("replicated"), do: "badge-success"
  defp status_badge("contested"), do: "badge-warning"
  defp status_badge("refuted"), do: "badge-error"
  defp status_badge(_), do: "badge-ghost"
end
