defmodule Web.Accounts.User do
  use Ecto.Schema
  import Ecto.Changeset

  schema "users" do
    field :email, :string
    field :name, :string
    field :google_uid, :string
    field :avatar, :string

    timestamps()
  end

  def changeset(user, attrs) do
    user
    |> cast(attrs, [:email, :name, :google_uid, :avatar])
    |> validate_required([:email, :google_uid])
    |> unique_constraint(:google_uid)
    |> unique_constraint(:email)
  end
end
