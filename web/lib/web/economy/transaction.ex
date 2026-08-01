defmodule Web.Economy.Transaction do
  use Ecto.Schema
  import Ecto.Changeset

  @types ~w(earn spend purchase withdraw reward referral fee)

  schema "transactions" do
    field :tx_type, :string
    field :amount, :float
    field :balance_after, :float
    field :description, :string
    field :reference_type, :string
    field :reference_id, :integer

    belongs_to :user, Web.Accounts.User

    timestamps()
  end

  def changeset(tx, attrs) do
    tx
    |> cast(attrs, [:tx_type, :amount, :balance_after, :description, :reference_type, :reference_id, :user_id])
    |> validate_required([:tx_type, :amount, :user_id])
    |> validate_inclusion(:tx_type, @types)
  end
end
