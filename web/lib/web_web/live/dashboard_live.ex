defmodule WebWeb.DashboardLive do
  use WebWeb, :live_view
  alias Web.Repo
  alias Web.Registry.Passport
  import Ecto.Query

  @impl true
  def mount(_params, session, socket) do
    user_email = session["user_email"]

    if is_nil(user_email) do
      {:ok, socket |> put_flash(:error, "Please log in first") |> redirect(to: ~p"/")}
    else
      passports =
        Repo.all(
          from p in Passport,
            where: p.user_email == ^user_email,
            order_by: [desc: p.updated_at]
        )

      {:ok,
       socket
       |> assign(:user_email, user_email)
       |> assign(:user_name, session["user_name"])
       |> assign(:passports, passports)
       |> assign(:selected_passport, nil)
       |> assign(:page_title, "Device Dashboard")}
    end
  end

  @impl true
  def render(assigns) do
    ~H"""
    <div class="min-h-screen bg-base-200">
      <div class="navbar bg-base-100 shadow-sm px-6">
        <div class="flex-1">
          <h1 class="text-xl font-bold">🛂 AIS Device Dashboard</h1>
        </div>
        <div class="flex-none gap-2">
          <span class="text-sm opacity-70"><%= @user_name || @user_email %></span>
          <a href="/auth/logout" class="btn btn-ghost btn-sm">Logout</a>
        </div>
      </div>

      <div class="max-w-5xl mx-auto p-4">
        <!-- Actions -->
        <div class="flex gap-2 mb-6">
          <a href="/passport/new" class="btn btn-primary">
            + New Passport
          </a>
          <span class="flex-1"></span>
          <div class="stats shadow">
            <div class="stat py-2 px-4">
              <div class="stat-title text-xs">Devices</div>
              <div class="stat-value text-lg"><%= length(@passports) %></div>
            </div>
          </div>
        </div>

        <!-- Passport Table -->
        <div class="card bg-base-100 shadow-xl">
          <div class="card-body p-0">
            <div class="overflow-x-auto">
              <table class="table table-zebra">
                <thead>
                  <tr>
                    <th>Device</th>
                    <th>Platform</th>
                    <th>Risk</th>
                    <th>Capabilities</th>
                    <th>Updated</th>
                    <th></th>
                  </tr>
                </thead>
                <tbody>
                  <%= for p <- @passports do %>
                    <tr class="hover">
                      <td>
                        <div class="font-bold"><%= p.name %></div>
                        <div class="text-xs opacity-50"><%= p.device_id |> String.slice(0, 8) %>...</div>
                      </td>
                      <td><span class="badge badge-ghost text-xs"><%= p.platform %></span></td>
                      <td><span class={"badge text-xs " <> risk_badge(p.risk_class)}><%= p.risk_class %></span></td>
                      <td>
                        <%= caps = get_in(p.payload || %{}, ["capabilities"]) || [] %>
                        <%= length(caps) %> caps
                      </td>
                      <td class="text-xs opacity-70">
                        <%= p.updated_at |> Calendar.strftime("%d %b %H:%M") %>
                      </td>
                      <td>
                        <button class="btn btn-ghost btn-xs" phx-click="view_passport" phx-value-id={p.id}>
                          View
                        </button>
                      </td>
                    </tr>
                  <% end %>
                  <%= if @passports == [] do %>
                    <tr>
                      <td colspan="6" class="text-center py-8 opacity-50">
                        No devices yet. <a href="/passport/new" class="link link-primary">Create your first passport</a>
                      </td>
                    </tr>
                  <% end %>
                </tbody>
              </table>
            </div>
          </div>
        </div>

        <!-- Selected Passport Detail -->
        <%= if @selected_passport do %>
          <div class="card bg-base-100 shadow-xl mt-6">
            <div class="card-body">
              <h2 class="card-title">
                <%= @selected_passport.name %>
                <span class={"badge " <> risk_badge(@selected_passport.risk_class)}><%= @selected_passport.risk_class %></span>
              </h2>
              <p class="text-sm opacity-70"><%= @selected_passport.description %></p>
              <div class="divider my-1"></div>

              <div class="grid grid-cols-3 gap-4 text-sm">
                <div>
                  <div class="font-bold text-xs uppercase opacity-50">Device ID</div>
                  <div class="font-mono text-xs"><%= @selected_passport.device_id %></div>
                </div>
                <div>
                  <div class="font-bold text-xs uppercase opacity-50">Platform</div>
                  <div><%= @selected_passport.platform %></div>
                </div>
                <div>
                  <div class="font-bold text-xs uppercase opacity-50">Registered</div>
                  <div><%= @selected_passport.inserted_at |> Calendar.strftime("%d %b %Y %H:%M") %></div>
                </div>
              </div>

              <div class="divider my-1"></div>

              <h3 class="font-bold text-sm">
                Capabilities (<%= caps_count(@selected_passport) %>)
              </h3>
              <div class="grid grid-cols-2 gap-2 text-xs">
                <%= for cap <- get_in(@selected_passport.payload || %{}, ["capabilities"]) || [] do %>
                  <div class="flex items-center gap-1 p-1 bg-base-200 rounded">
                    <span class={"badge badge-xs " <> risk_badge(cap["risk"] || "low")}></span>
                    <span><%= cap["name"] %></span>
                    <span class="opacity-50">(<%= length(cap["parameters"] || []) %> params)</span>
                  </div>
                <% end %>
              </div>

              <div class="divider my-1"></div>

              <h3 class="font-bold text-sm">
                Forbidden Always (<%= forbids_count(@selected_passport) %>)
              </h3>
              <div class="space-y-1 text-xs">
                <%= for f <- get_in(@selected_passport.payload || %{}, ["forbidden_always"]) || [] do %>
                  <div class="flex items-start gap-2">
                    <span class={"badge badge-xs mt-0.5 " <> if(f["constitutional"], do: "badge-error", else: "badge-warning")}>
                      <%= if(f["constitutional"], do: "CONST", else: "soft") %>
                    </span>
                    <div>
                      <strong><%= f["name"] %></strong>
                      <span class="opacity-70"> — <%= String.slice(f["reason"] || "", 0, 100) %></span>
                    </div>
                  </div>
                <% end %>
              </div>

              <div class="card-actions justify-end mt-2">
                <button class="btn btn-sm btn-ghost" phx-click="close_detail">Close</button>
              </div>
            </div>
          </div>
        <% end %>
      </div>
    </div>
    """
  end

  @impl true
  def handle_event("view_passport", %{"id" => id}, socket) do
    passport = Repo.get!(Passport, id)
    {:noreply, assign(socket, :selected_passport, passport)}
  end

  @impl true
  def handle_event("close_detail", _params, socket) do
    {:noreply, assign(socket, :selected_passport, nil)}
  end

  defp risk_badge("critical"), do: "badge-error"
  defp risk_badge("high"), do: "badge-error"
  defp risk_badge("medium"), do: "badge-warning"
  defp risk_badge("low"), do: "badge-success"
  defp risk_badge("informational"), do: "badge-ghost"
  defp risk_badge(_), do: "badge-ghost"

  defp caps_count(p), do: length(get_in(p.payload || %{}, ["capabilities"]) || [])
  defp forbids_count(p), do: length(get_in(p.payload || %{}, ["forbidden_always"]) || [])
end
