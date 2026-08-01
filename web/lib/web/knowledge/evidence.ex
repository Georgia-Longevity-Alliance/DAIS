defmodule Web.Knowledge.Evidence do
  use Ecto.Schema
  import Ecto.Changeset

  @types ~w(supporting contradicting inconclusive)

  schema "evidence_items" do
    field :evidence_type, :string, default: "supporting"
    field :strength, :float, default: 0.5
    field :description, :string

    belongs_to :claim, Web.Knowledge.Claim
    belongs_to :source, Web.Knowledge.Source
    belongs_to :user, Web.Accounts.User

    timestamps()
  end

  def changeset(evidence, attrs) do
    evidence
    |> cast(attrs, [:evidence_type, :strength, :description, :claim_id, :source_id, :user_id])
    |> validate_required([:claim_id, :user_id])
    |> validate_inclusion(:evidence_type, @types)
    |> validate_number(:strength, greater_than_or_equal_to: 0.0, less_than_or_equal_to: 1.0)
  end
end
