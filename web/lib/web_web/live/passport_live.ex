defmodule WebWeb.PassportLive do
  use WebWeb, :live_view

  @impl true
  def mount(_params, session, socket) do
    {_greeting, state} = Web.Interview.start()

    {:ok,
     socket
     |> assign(:interview, state)
     |> assign(:user_input, "")
     |> assign(:done, false)
     |> assign(:passport_json, nil)
     |> assign(:user_email, session["user_email"])
     |> assign(:user_id, session["user_id"])}
  end

  @impl true
  def render(assigns) do
    ~H"""
    <div class="min-h-screen bg-base-200">
      <div class="navbar bg-base-100 shadow-sm px-6">
        <div class="flex-1">
          <h1 class="text-xl font-bold">🛂 AISocket Passport Interview</h1>
        </div>
        <div class="flex-none gap-2">
          <div class="badge badge-ghost text-xs"><%= @interview.phase %></div>
          <div class="badge badge-neutral text-xs">Q:<%= @interview.question_count %></div>
        </div>
      </div>

      <div class="max-w-3xl mx-auto px-4 pb-28 pt-4">
        <div id="chat-messages" class="space-y-4" phx-hook="ScrollToBottom">
          <%= for {msg, i} <- Enum.with_index(@interview.messages) do %>
            <div class={"chat " <> if(msg.role == "interviewer", do: "chat-start", else: "chat-end")}>
              <div class={"chat-bubble max-w-prose whitespace-pre-line text-sm " <> if(msg.role == "interviewer", do: "chat-bubble-primary", else: "chat-bubble-neutral")}>
                <%= msg.text %>
              </div>
            </div>
          <% end %>

          <%= if @done do %>
            <div class="alert alert-success shadow-lg mt-6">
              <svg xmlns="http://www.w3.org/2000/svg" class="stroke-current shrink-0 h-6 w-6" fill="none" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z" /></svg>
              <div>
                <h3 class="font-bold">✅ Passport Created!</h3>
                <div class="text-xs">Your device passport is ready.</div>
              </div>
              <div class="flex gap-2">
                <button class="btn btn-sm btn-ghost" phx-click="show_json">View JSON</button>
                <button class="btn btn-sm btn-primary" phx-click="register_passport">Register</button>
              </div>
            </div>
          <% end %>

          <%= if @passport_json do %>
            <div class="mockup-code mt-4">
              <pre class="text-xs p-4 max-h-96 overflow-auto"><code><%= @passport_json %></code></pre>
            </div>
          <% end %>
        </div>
      </div>

      <div class="fixed bottom-0 left-0 right-0 bg-base-100 border-t border-base-300 p-4">
        <div class="max-w-3xl mx-auto">
          <%= if !@done do %>
            <form phx-submit="send_message" class="flex gap-2">
              <input
                type="text"
                name="message"
                value={@user_input}
                placeholder="Type your answer..."
                class="input input-bordered flex-1"
                autocomplete="off"
                autofocus
                id="message-input"
                phx-hook="FocusInput"
              />
              <button type="submit" class="btn btn-primary">
                Send
              </button>
            </form>
          <% else %>
            <div class="flex gap-2 justify-center">
              <button class="btn btn-outline btn-sm" phx-click="restart">
                Start New Interview
              </button>
              <button class="btn btn-primary btn-sm" phx-click="register_passport">
                Register Passport
              </button>
            </div>
          <% end %>
        </div>
      </div>
    </div>
    """
  end

  @impl true
  def handle_event("send_message", %{"message" => text}, socket) do
    text = String.trim(text)

    if text == "" do
      {:noreply, socket}
    else
      io = socket.assigns.interview

      case Web.Interview.next(io, text) do
        {_reply, new_state} ->
          done = Web.Interview.done?(new_state)

          socket =
            socket
            |> assign(:interview, new_state)
            |> assign(:user_input, "")
            |> assign(:done, done)

          if done do
            passport = Web.Interview.build_passport(new_state)
            json = Jason.encode!(passport, pretty: true)
            {:noreply, assign(socket, :passport_json, json)}
          else
            {:noreply, push_event(socket, "scroll_bottom", %{})}
          end
      end
    end
  end

  @impl true
  def handle_event("show_json", _params, socket) do
    passport = Web.Interview.build_passport(socket.assigns.interview)
    json = Jason.encode!(passport, pretty: true)
    {:noreply, assign(socket, :passport_json, json)}
  end

  @impl true
  def handle_event("register_passport", _params, socket) do
    passport = Web.Interview.build_passport(socket.assigns.interview)
    user_email = socket.assigns[:user_email] || "anonymous@local"
    user_id = socket.assigns[:user_id]

    case Web.Repo.insert(%Web.Registry.Passport{
      device_id: passport["device_id"],
      name: passport["name"],
      description: passport["description"],
      platform: passport["platform"],
      risk_class: passport["risk_class"],
      payload: passport,
      user_id: user_id || passport["device_id"],
      user_email: user_email
    }) do
      {:ok, _record} ->
        {:noreply,
         socket
         |> put_flash(:info, "✅ Passport registered locally!")
         |> push_event("scroll_bottom", %{})}

      {:error, changeset} ->
        {:noreply,
         socket
         |> put_flash(:error, "Registration failed: #{inspect(changeset.errors)}")}
    end
  end

  @impl true
  def handle_event("restart", _params, socket) do
    {_greeting, state} = Web.Interview.start()

    {:noreply,
     socket
     |> assign(:interview, state)
     |> assign(:user_input, "")
     |> assign(:done, false)
     |> assign(:passport_json, nil)
     |> push_event("scroll_bottom", %{})}
  end
end
