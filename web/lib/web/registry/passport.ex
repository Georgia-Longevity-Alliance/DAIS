defmodule Web.Registry.Passport do
  use Ecto.Schema
  import Ecto.Changeset

  schema "passports" do
    field :device_id, :string
    field :name, :string
    field :description, :string
    field :platform, :string
    field :risk_class, :string
    field :payload, :map    # Full JSON
    field :user_id, :string
    field :user_email, :string

    timestamps()
  end

  def changeset(passport, attrs) do
    passport
    |> cast(attrs, [:device_id, :name, :description, :platform, :risk_class, :payload, :user_id, :user_email])
    |> validate_required([:device_id, :name, :payload])
  end
end
