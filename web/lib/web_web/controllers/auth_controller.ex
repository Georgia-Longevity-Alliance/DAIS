defmodule WebWeb.AuthController do
  use WebWeb, :controller
  alias Web.Accounts.User
  alias Web.Repo

  def request(conn, _params) do
    # Redirect to Google OAuth
    redirect(conn, to: ~p"/auth/google")
  end

  def callback(%{assigns: %{ueberauth_failure: _failure}} = conn, _params) do
    conn
    |> put_flash(:error, "Authentication failed")
    |> redirect(to: ~p"/")
  end

  def callback(%{assigns: %{ueberauth_auth: auth}} = conn, _params) do
    user = upsert_user!(auth)

    conn
    |> put_session(:user_id, user.id)
    |> put_session(:user_email, user.email)
    |> put_session(:user_name, user.name)
    |> put_flash(:info, "Welcome, #{user.name || user.email}!")
    |> redirect(to: ~p"/dashboard")
  end

  def logout(conn, _params) do
    conn
    |> clear_session()
    |> put_flash(:info, "Logged out")
    |> redirect(to: ~p"/")
  end

  defp upsert_user!(auth) do
    info = auth.info
    uid = auth.uid
    email = info.email
    name = info.name
    avatar = List.first(info.urls["photos"] || [])

    case Repo.get_by(User, google_uid: uid) do
      nil ->
        %User{}
        |> User.changeset(%{
          google_uid: uid,
          email: email,
          name: name,
          avatar: avatar
        })
        |> Repo.insert!()

      user ->
        user
        |> User.changeset(%{email: email, name: name, avatar: avatar})
        |> Repo.update!()
    end
  end
end
