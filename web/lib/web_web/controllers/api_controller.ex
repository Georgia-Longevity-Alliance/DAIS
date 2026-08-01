defmodule WebWeb.ApiController do
  use WebWeb, :controller
  alias Web.Repo
  alias Web.Registry.Passport
  alias Web.Knowledge.Claim
  alias Web.Economy.{Listing, Transaction, Engine}
  alias Web.Accounts.User
  import Ecto.Query

  def health(conn, _params) do
    json(conn, %{status: "ok", service: "AIS local registry"})
  end

  def register_passport(conn, %{"passport" => passport_data} = params) do
    user_email = params["user_email"] || "anonymous@local"

    # Validate the passport structure
    case validate_passport(passport_data) do
      :ok ->
        passport =
          %Passport{}
          |> Passport.changeset(%{
            device_id: passport_data["device_id"],
            name: passport_data["name"],
            description: passport_data["description"],
            platform: passport_data["platform"],
            risk_class: passport_data["risk_class"],
            payload: passport_data,
            user_id: params["user_id"] || passport_data["device_id"],
            user_email: user_email
          })
          |> Repo.insert!()

        conn
        |> put_status(:created)
        |> json(%{
          status: "registered",
          device_id: passport.device_id,
          name: passport.name,
          url: ~p"/api/passports/#{passport.id}"
        })

      {:error, reason} ->
        conn
        |> put_status(:unprocessable_entity)
        |> json(%{error: reason})
    end
  end

  def register_passport(conn, _params) do
    conn
    |> put_status(:bad_request)
    |> json(%{error: "Missing 'passport' field in request body"})
  end

  def list_passports(conn, params) do
    user_email = params["user_email"]

    query =
      if user_email do
        from p in Passport, where: p.user_email == ^user_email, order_by: [desc: p.updated_at]
      else
        from p in Passport, order_by: [desc: p.updated_at]
      end

    passports =
      Repo.all(query)
      |> Enum.map(&passport_summary/1)

    json(conn, %{passports: passports, count: length(passports)})
  end

  def get_passport(conn, %{"id" => id}) do
    case Repo.get(Passport, id) do
      nil ->
        conn
        |> put_status(:not_found)
        |> json(%{error: "Passport not found"})

      passport ->
        json(conn, passport.payload)
    end
  end

  defp passport_summary(p) do
    %{
      id: p.id,
      device_id: p.device_id,
      name: p.name,
      platform: p.platform,
      risk_class: p.risk_class,
      capabilities_count: length(get_in(p.payload || %{}, ["capabilities"]) || []),
      updated_at: p.updated_at
    }
  end

  defp validate_passport(data) when is_map(data) do
    cond do
      is_nil(data["name"]) or data["name"] == "" ->
        {:error, "Passport must have a 'name'"}

      is_nil(data["device_id"]) ->
        {:error, "Passport must have a 'device_id'"}

      true ->
        :ok
    end
  end

  # ── Knowledge Graph API ──

  def list_claims(conn, _params) do
    claims = Repo.all(from c in Claim, order_by: [desc: c.updated_at], limit: 100, preload: [:user])
    json(conn, %{
      claims: Enum.map(claims, &claim_json/1),
      count: length(claims)
    })
  end

  def create_claim(conn, params) do
    user = get_user(params)

    case Repo.insert(Claim.changeset(%Claim{}, %{
      subject: params["subject"],
      predicate: params["predicate"],
      object: params["object"],
      domain: params["domain"],
      user_id: user.id
    })) do
      {:ok, claim} ->
        claim = Repo.preload(claim, :user)
        Engine.award_contribution(user, claim, 0.5)
        conn |> put_status(:created) |> json(claim_json(claim))
      {:error, cs} ->
        conn |> put_status(:unprocessable_entity) |> json(%{error: inspect(cs.errors)})
    end
  end

  def get_claim(conn, %{"id" => id}) do
    claim = Repo.get!(Claim, id) |> Repo.preload([:user, :evidence_items])
    json(conn, claim_json(claim))
  end

  # ── Marketplace API ──

  def list_listings(conn, _params) do
    listings = Repo.all(from l in Listing, where: l.active == true, order_by: [desc: l.updated_at], preload: [:user], limit: 100)
    json(conn, %{
      listings: Enum.map(listings, &listing_json/1),
      count: length(listings)
    })
  end

  def create_listing(conn, params) do
    user = get_user(params)

    case Repo.insert(Listing.changeset(%Listing{}, %{
      title: params["title"],
      description: params["description"],
      price_credits: params["price_credits"],
      content_type: params["content_type"],
      user_id: user.id
    })) do
      {:ok, listing} ->
        listing = Repo.preload(listing, :user)
        conn |> put_status(:created) |> json(listing_json(listing))
      {:error, cs} ->
        conn |> put_status(:unprocessable_entity) |> json(%{error: inspect(cs.errors)})
    end
  end

  def buy_listing(conn, %{"id" => id}) do
    user = get_user(conn.params)
    listing = Repo.get!(Listing, id)

    case Engine.purchase_listing(user, listing) do
      {:ok, listing} ->
        json(conn, %{status: "purchased", listing: listing_json(listing)})
      {:error, :insufficient_credits, needed: needed} ->
        conn |> put_status(:payment_required) |> json(%{error: "insufficient_credits", credits_needed: needed})
      {:error, reason} ->
        conn |> put_status(:unprocessable_entity) |> json(%{error: inspect(reason)})
    end
  end

  # ── Economy API ──

  def economy_stats(conn, _params) do
    json(conn, Engine.stats())
  end

  def list_transactions(conn, params) do
    user = get_user(params)
    txs = Repo.all(from t in Transaction, where: t.user_id == ^user.id, order_by: [desc: t.inserted_at], limit: 100)
    json(conn, %{transactions: Enum.map(txs, &tx_json/1), count: length(txs)})
  end

  # ── JSON helpers ──

  defp claim_json(c) do
    user_info = case Map.get(c, :user) do
      %{name: n, email: e} -> %{name: n, email: e}
      _ -> nil
    end
    %{
      id: c.id,
      subject: c.subject,
      predicate: c.predicate,
      object: c.object,
      status: c.status,
      domain: c.domain,
      confidence: c.confidence,
      reward_paid: c.reward_paid,
      user: user_info,
      inserted_at: c.inserted_at
    }
  end

  defp listing_json(l) do
    user_info = case Map.get(l, :user) do
      %{name: n, email: e} -> %{name: n, email: e}
      _ -> nil
    end
    %{
      id: l.id,
      title: l.title,
      description: l.description,
      price_credits: l.price_credits,
      content_type: l.content_type,
      sales_count: l.sales_count,
      user: user_info,
      inserted_at: l.inserted_at
    }
  end

  defp tx_json(t) do
    %{
      id: t.id,
      tx_type: t.tx_type,
      amount: t.amount,
      balance_after: t.balance_after,
      description: t.description,
      inserted_at: t.inserted_at
    }
  end

  defp get_user(params) do
    user_email = params["user_email"] || "anonymous@local"
    case Repo.get_by(User, email: user_email) do
      nil ->
        # Auto-create anonymous user
        %User{email: user_email, google_uid: "anon_#{user_email}"}
        |> User.changeset(%{})
        |> Repo.insert!()
      user -> user
    end
  end
end
