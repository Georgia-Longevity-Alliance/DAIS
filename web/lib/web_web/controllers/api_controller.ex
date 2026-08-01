defmodule WebWeb.ApiController do
  use WebWeb, :controller
  alias Web.Repo
  alias Web.Registry.Passport
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
end
