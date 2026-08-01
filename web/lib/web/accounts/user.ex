defmodule Web.Accounts.User do
  use Ecto.Schema
  import Ecto.Changeset

  @roles ~w(contributor consumer admin)

  schema "users" do
    field :email, :string
    field :name, :string
    field :google_uid, :string
    field :avatar, :string
    field :credits_balance, :float, default: 0.0
    field :credits_earned_total, :float, default: 0.0
    field :credits_spent_total, :float, default: 0.0
    field :role, :string, default: "contributor"
    field :reputation_score, :float, default: 0.0
    field :contributions_count, :integer, default: 0

    has_many :claims, Web.Knowledge.Claim
    has_many :transactions, Web.Economy.Transaction
    has_many :listings, Web.Economy.Listing

    timestamps()
  end

  def changeset(user, attrs) do
    user
    |> cast(attrs, [:email, :name, :google_uid, :avatar, :credits_balance, :role])
    |> validate_required([:email, :google_uid])
    |> validate_inclusion(:role, @roles)
    |> unique_constraint(:google_uid)
    |> unique_constraint(:email)
  end
end
